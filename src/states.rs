use std::sync::atomic::{AtomicI64, Ordering};
use std::{ffi::OsString, path::Path};

use sqlite::{Connection, State, Value};

#[derive(Default)]
pub struct SqliteState {
    db_path: OsString,
    token: String,
    auth_chat_id: AtomicI64,
}

impl SqliteState {
    pub fn new(path: OsString, token: String) -> Result<SqliteState, String> {
        let mut is_create_tables = false;
        if !Path::new(&path).exists() {
            is_create_tables = true;
        }

        let connection = match Connection::open_thread_safe(&path) {
            Ok(c) => c,
            Err(e) => {
                return Err(e.message.unwrap());
            }
        };

        if is_create_tables {
            let query =
                "CREATE TABLE users (chat_id INTEGER PRIMARY KEY, current_path TEXT NOT NULL);";
            connection.execute(query).unwrap();

            Ok(SqliteState {
                db_path: path.clone(),
                token,
                auth_chat_id: AtomicI64::new(0),
            })
        } else {
            let query = "SELECT * FROM users LIMIT 1;";
            let auth_chat_id = AtomicI64::new(0);
            connection
                .iterate(query, |pairs| {
                    for &(name, value) in pairs.iter() {
                        if "chat_id" == name {
                            auth_chat_id
                                .store(value.unwrap().parse::<i64>().unwrap(), Ordering::Relaxed);
                        }
                    }
                    true
                })
                .unwrap();

            Ok(SqliteState {
                db_path: path.clone(),
                token,
                auth_chat_id,
            })
        }
    }

    fn replace_into_auth_chat_id(&self, chat_id: i64) {
        let connection = Connection::open_thread_safe(&self.db_path).unwrap();

        let query = "REPLACE INTO users VALUES (:chat_id, :current_path)";
        let mut statement = connection.prepare(query).unwrap();

        statement
            .bind_iter::<_, (_, Value)>([
                (":chat_id", chat_id.into()),
                (":current_path", "/".into()),
            ])
            .unwrap();

        while let Ok(State::Row) = statement.next() {}
    }

    pub fn update_current_path(&self, path: &OsString) {
        let connection = Connection::open_thread_safe(&self.db_path).unwrap();

        let query = "UPDATE users SET current_path=:current_path";
        let mut statement = connection.prepare(query).unwrap();

        statement
            .bind_iter::<_, (_, Value)>([(
                ":current_path",
                path.to_owned().into_string().unwrap().into(),
            )])
            .unwrap();

        while let Ok(State::Row) = statement.next() {}
    }

    pub fn query_current_path(&self, chat_id: i64) -> Option<OsString> {
        let connection = Connection::open_thread_safe(&self.db_path).unwrap();

        let mut ret_os_string = None;
        let query = "SELECT current_path FROM users WHERE chat_id=?";
        for row in connection
            .prepare(query)
            .unwrap()
            .into_iter()
            .bind((1, chat_id))
            .unwrap()
            .map(|row| row.unwrap())
        {
            ret_os_string = Some(OsString::from(row.read::<&str, _>("current_path")));
            break;
        }

        ret_os_string
    }

    pub fn get_auth_token(&self) -> String {
        self.token.clone()
    }

    pub fn set_auth_chat_id(&self, chat_id: i64) {
        self.replace_into_auth_chat_id(chat_id);
        self.auth_chat_id.store(chat_id, Ordering::Relaxed);
    }

    pub fn get_auth_chat_id(&self) -> Option<i64> {
        let v = self.auth_chat_id.load(Ordering::Relaxed);
        if v != 0 {
            Some(v)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_uses() {
        let token = "test token xxx".to_string();
        let ss = SqliteState::new("./test.db".into(), token.clone()).unwrap();
        assert_eq!(ss.token, token);

        let chatid = 2_991_384_423i64;
        ss.set_auth_chat_id(chatid);
        assert_eq!(ss.get_auth_chat_id().unwrap(), chatid);
        let ospath = OsString::from("/home/x");
        ss.update_current_path(&ospath);

        let cp = ss.query_current_path(chatid).unwrap();
        assert_eq!(cp, ospath);
    }
}
