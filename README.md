# Translate

基于Tauri的划词翻译软件

## Todo

- [x] 划词翻译
- [x] markdown与公式渲染
- [x] 文档
- [x] 配置文件重载
- [ ] 开机自启动
- [ ] 更加完善的错误处理
- [ ] 更多api支持
- [ ] 自定义主题





## 使用教程

### 设置apikey

第一次打开时需要配置模型apikey，目前支持了deepseek和doubao。

[[申请deepseek api]](https://platform.deepseek.com/api_keys)    [[申请火山引擎api]](https://console.volcengine.com/ark/region:ark+cn-beijing/apiKey)

目前火山引擎会赠送50万token的额度，可以试用一下。


### 快捷键

目前划词翻译绑定弹窗为\<C-Capslock\>，选中文字后双击即可进行翻译；

可调整窗口大小；

按住Alt键可鼠标拖动翻译弹窗调整位置；

### 设置

打开配置文件：系统托盘 - Config

启用/关闭快捷键：系统托盘 - enable/disable shortcut

完全退出应用: 系统托盘 - Quit

