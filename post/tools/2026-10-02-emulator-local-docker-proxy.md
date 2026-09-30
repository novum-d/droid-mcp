【📰 News投稿】  
Android EmulatorからローカルのDocker環境へHTTPS接続する方法  
────────────────────  
Mac上のHTTPSプロキシを経由させ、ホスト側の`/etc/hosts`で名前解決することで、Android EmulatorやiOS SimulatorからローカルのDocker Compose環境へ本番と同じホスト名で接続する方法です。AndroidではデバッグビルドのNetwork Security Configurationでユーザー追加CAを信頼させます。  
💬 ── コメント ──  
複数ホストをリバースプロキシで振り分ける開発環境でも、アプリの接続先やCookieの条件を本番に近づけられる点が参考になります。独自CAを使うため、対象OSとtargetSdkの証明書要件は別途確認が必要です。  
🔗 ── 参考URL ──  
https://proandroiddev.com/proxy-your-android-emulator-and-ios-simulator-to-your-local-docker-stack-d251321d8017
