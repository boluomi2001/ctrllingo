# Windows 隔离验收

Python 3，仅使用标准库。只向本机 127.0.0.1 模拟服务发送请求，不注册输入法、不调用真实账户。

先构建 release 主程序和设置，按数据锁取得产品数据。运行：

```powershell
$env:NO_PROXY='localhost,127.0.0.1,::1'
python tools/windows-tests/smoke.py --binary-dir target/release --output-dir target/validation/smoke
```

该脚本复制程序与数据到独立暂存目录，设置隔离 APPDATA/LOCALAPPDATA/TEMP。测试五语言按需请求、过期结果、缓存、费用账本与费用卡片。不要与本机已运行的译点同时执行，也不要并行运行两份脚本；共享的产品命名管道一次只能由一份服务监听。

`ui.py --stage <隔离程序与数据目录> --output-dir target/validation/ui` 测试费用窗口消息级交互，并核对目标 Server 与 target/release 的 SHA-256。它以 PostMessage 发送窗口消息，不代表真实物理点击或 TSF 上屏验收。

失败报告和截图留在 target/validation 下供复核。不要将个人配置、密钥或真实输入日志提交到仓库。
