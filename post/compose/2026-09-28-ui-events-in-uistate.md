【🧩 Compose投稿】  
Composeの一回限りのUIイベントをUiStateで扱う3段階の設計  
────────────────────  
Snackbar、画面遷移、ダイアログなどの一回限りのイベントを、単一の`UiState`で扱う実装例です。単一のnullableなイベントから始め、複数イベントが蓄積する場合はキューへ移行し、消費対象を明確にする必要が生じた段階で各イベントへIDを付ける3段階が示されています。  
💬 ── サンプル・補足 ──  
最初から複雑なイベント管理を導入せず、同時発生や再実行といった要件が現れた時点で仕組みを増やす判断例として参考になりそうです。  
🔗 ── 参考URL ──  
https://proandroiddev.com/ui-events-in-uistate-start-simple-add-a-queue-only-when-you-need-one-5b59dac0261e
