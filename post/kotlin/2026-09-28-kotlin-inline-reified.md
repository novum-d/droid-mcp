【📰 News投稿】  
Kotlinのinlineとreifiedが型消去を補う仕組み  
────────────────────  
JVMではジェネリック型の情報が実行時に消去されますが、Kotlinでは関数を呼び出し元へ展開する`inline`と、型を具体化する`reified`を組み合わせることで、`T::class`や`x is T`のような処理を記述できます。インライン展開された関数がコードカバレッジ上で追跡されにくい点にも触れています。  
💬 ── コメント ──  
JSON変換やDIなどで見かける型安全なAPIが、どのようにJVMの型消去を補っているのかを整理する材料になりそうです。  
🔗 ── 参考URL ──  
https://proandroiddev.com/why-kotlins-inline-and-reified-keywords-exist-and-how-they-solve-java-s-oldest-flaw-f390508339cd
