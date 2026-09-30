【📰 News投稿】  
LiteRT-LMで端末内LLMを動かす実装例と注意点  
────────────────────  
LiteRT-LMを使ったAndroidアプリの実装例です。モデルを保持するEngineと会話状態を持つSessionを使い、応答のストリーミング、CPU/GPUの選択、マルチモーダル入力を扱っています。入力と出力を合わせたトークン枠の設定や、LiteRTとのネイティブライブラリ競合にも触れています。  
💬 ── コメント ──  
端末内で動くチャット機能を検討する際、モデルの実行方法に加えて、メモリ使用量とビルド時のライブラリ構成を確認する材料になりそうです。  

Pixel標準のGemini Nanoとは異なり、LiteRT / LiteRT-LMは開発者のカスタムAIをアプリに組み込めます。
🔗 ── 参考URL ──  
<https://proandroiddev.com/on-device-ai-series-part-5-litert-lm-d6c23b102094>
