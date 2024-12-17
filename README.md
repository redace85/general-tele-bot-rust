# general-tele-bot-rust
general purpose telegrambot written with rust

using teloxide
a remote bot


telebot.service
```
[Unit]
Description=telebot service
After=network.target
StartLimitIntervalSec=0

[Service]
Type=simple
Restart=always
RestartSec=1
User=redace85
WorkingDirectory=/home/redace85/TeleBotDir
ExecStart=/home/redace85/TeleBotDir/general-tele-bot-rust

[Install]
WantedBy=multi-user.target

```
