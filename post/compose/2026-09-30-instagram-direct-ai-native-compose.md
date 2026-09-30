【🧩 Compose投稿】  
Instagram DirectがComposeでAI向けのUIアーキテクチャを構築  
────────────────────  
Instagram Directは、既存のViewベースUIをJetpack Composeへ段階的に移行し、AIエージェントが変更しやすい構造へ再設計しました。Metaの内部分析では、移行済み画面のコード量が約50%減り、Viewで同様の作業を行う場合と比べて、エージェントの実行時間が35%、エンジニアとのやり取りが32%、セッション当たりのトークンコストが33%減少しています。  
💬 ── サンプル・補足 ──  
効果はInstagram Directの内部測定結果です。ViewとComposeを細かく混在させる期間を抑え、状態やコンポーネントの境界をアーキテクチャで明確にした点が参考になります。  
🔗 ── 参考URL ──  
https://developer.android.com/blog/posts/jetpack-compose-ai-native-ui-instagram-direct
