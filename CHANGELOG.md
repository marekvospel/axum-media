
## 0.2.0 (2023-11-23)

* ⚠️ BREAKING: add field to `AnyMedia` to use when decoding
* ⚠️ BREAKING: remove `AnyMediaIntoResponse`
* fix: add readme and licenses to axum-media crate
* feat: implement `Deref<Target = T>` for `AnyMedia<T>`
* feat: implement `From<T>` for `AnyMedia<T>`
* feat: add `Accept` extractor
* docs: document the extractors, rejection and features
