# m3u8_download

一个超级简单的m3u8下载例子

需要安装:

```shell
apt install ffmpeg
apt install aria2
```

接口:

```
http://127.0.0.1:9090/download?user=用户ID&url=m3u8视频url&name=视频名字&file=第一集
```

## 创建服务

`/etc/systemd/system/app.service` 内容:

```
[Unit]
Description=app
After=network.target

[Service]
ExecStart=/root/app
Restart=always
User=root
Group=root

[Install]
WantedBy=multi-user.target
```

`sudo systemctl enable app.service` 确保开机自动启动.

`systemctl status app.service` 查看状态.
