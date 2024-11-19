
# Relase note

## 0.16.0

- Remove some unused struct

- Remove implementations of Infer for reference type

- Remove Alter

- Add *_map and *_mutable to Infer

- Improve the implementation of Infer and Fetch

## 0.14.0

- Remove RawVal, AStr

- Remove extractor pattern

- Update neure version

## 0.12.1

- Error improvement

## 0.12.0

### aopt

- Remove `utf8` feature, using `OsStr` by default. 

- Change the type `Str` to `AStr`.

- Imporve the option build api, change the `Config` trait to `ConfigBuild` trait.
This affects all APIs related to adding adding and searching `Opt`s.
Such as we removed all the APIs ending with `_i` from `OptSet`, the `add_opt_i::<T>(?)` now become `add_opt(?.infer::<T>())`.

- Remove some APIs from `Args`, using `Args::from` instead of `Args::from_array`.

### cote

- Using `cote::prelude::*` instead of `cote::*`.

- Refactor the `cote-derive`.

- Improve the `Fetch` implementation, using `Uid` instead of `&str` improve the performance.