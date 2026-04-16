# Translate

基于Tauri的划词翻译软件

## Todo

- [x] 划词翻译
- [x] markdown与公式渲染
- [x] 文档
- [x] 配置文件重载
- [x] 开机自启动
- [ ] 更加完善的错误处理
- [ ] 更多api支持
- [ ] 自定义主题





## 使用教程

### 设置apikey

第一次打开时需要配置模型apikey，目前支持了deepseek和doubao。

[[申请deepseek api]](https://platform.deepseek.com/api_keys)    [[申请火山引擎api]](https://console.volcengine.com/ark/region:ark+cn-beijing/apiKey)

目前火山引擎会赠送50万token的额度，可以试用一下。


### 快捷键

划词翻译快捷键支持通过配置文件自定义，默认是 `Alt+W`。

配置路径：`~/.local/translate/config.toml`

配置项：

```toml
[shortcuts]
trigger_translation = "Alt+W"
enabled_by_default = true
```

修改配置后，在系统托盘点击 `Reload Shortcut` 立即生效（无需重启）。

可调整窗口大小；

按住Alt键可鼠标拖动翻译弹窗调整位置；

### 设置

打开配置文件：系统托盘 - Config

启用/关闭快捷键：系统托盘 - enable/disable shortcut

重载快捷键配置：系统托盘 - Reload Shortcut

完全退出应用: 系统托盘 - Quit

