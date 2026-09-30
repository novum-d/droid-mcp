【📰 News投稿】  
Kotlin Flowの`buffer()`と`flowOn()`の融合が途切れる条件  
────────────────────  
Kotlin Flowの`buffer()`と`flowOn()`を隣接させるとチャネルが融合します。筆者の実測では、間に`map()`などを挟むと追加のチャネルやコルーチンが生じ、`conflate()`では融合が維持されました。  

💬 ── コメント ──  
高頻度のFlowで処理負荷が気になるときは、`buffer()`と`flowOn()`の配置を確認したいです。  
🔗 ── 参考URL ──  
https://proandroiddev.com/i-counted-every-hidden-channel-kotlin-flow-creates-most-devs-are-wrong-about-which-operators-are-8f38cf676e07
