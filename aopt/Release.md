
# Relase note

## 0.9.11

- Change the 'static of Invoker to 'a

- Remove the into_boxed of Parser

- Improve `getopt!`

- Improve the handler register for option

- Improve the infer information, option using same type_id for T, Option<T>, Vec<T>, Option<Vec<T>>

- Fix the PrePolicy, don't ignore the failure from `Checker`

- Change the name of `wrap_handler_fallback` to `wrap_handler_fallback_action`

- Add a new handler wrapper function `wrap_handler_fallback`

## 0.9.1

- Add type paramter for `Main`, `Any`

- Change the default value of `Cmd`, `Pos` to `bool`

- Change the guess argument of `Cmd` to `true`.

- Remove `Noa` type

## 0.9.0

- Moving invoke service from ser/ to ctx/

- Change the interface of `Policy`

- Remove return value of `Parser`

- Add a style manager manage the `UserStyle`.

- Change the NOA index base on 0, pass first command line argument to `Main`

- Improve the api of ReturnVal, fix the `getopt!` return value

- Improve the option creator implemetation

- Let option callback return `bool` instead of `Option`

- Make tracing log optional

- Make serde optional

- Remove type from parser, add alias and help message

- Support create option from type and string both

- Fix filter match issue

## 0.8.4

- Add `Creator::any` option type

- Fix the OptConfig with_* functions

- Add `Parser::add_opt_cfg` function add option by configurations

- Fix the prefix issue of `EmbeddedValue` style and `CombinedOption` style

- Add `EmbeddedValue` support option set like `--opt42`

## 0.8.3

- Add Into<Args> for ReturnVal

## 0.8.2

- Fix the compile error on linux

## 0.8.0

- Removing the prefix of option

- Change the optional to force of option

- Remove the deactivate style '/' from create string

## 0.7.4

- Moving the index parsing logical from StrParser to Index

## 0.7.3

- Fix a issue when using Parser inside async function

- Remove the initialize code inside Policy::parse, call it outside

- Fix getopt!, add initialize for option

## 0.7.1

- Fix a backward POS check issue

## 0.7.0

- refactor whole project

## 0.6.7

- fix the noa process issue

## 0.6.6

- update documents

- fix the display of option index

## 0.6.5

- fix documents

- fix clippy

- fix the parser of option set string

- update the parser test case

## 0.6.2

- add index parameter when parsing option value, not affect exist code logical

- update aopt-macro version

## 0.6.1

- add policy change api to Parser and DynParser

- add take_* to Value

- change meger_* of Value

- add reset to Parser

- implement Sync/Send for Parser

## 0.6.0

- refactor set as type parameter

- refactor index access of SimpleSet

- change the nonopt_check to cmd_check and pos_check

- fix the check logical of CMD POS and MAIN

## 0.5.8

- update documents

- add some api to OptValue

## 0.5.7

- change the alias hint character

- fix pos force required error display

- fix callback commit issue caused by *DerefMut*

## 0.5.6

- fix test in documents

## 0.5.5

- add get_strict api to policy

- update documents

- change the name of Callback type

## 0.5.4

- fix aopt-macro dependence

- add sync feature

- make Service::invoke paramter self mutable

- add a callback store type

- remove callback api from SingleApp

## 0.5.3

- change dyn Set to type parameter of Matcher

## 0.5.2

- add macro getopt/getoptd

- refactor the parser

- refactor test case

- refactor SingleApp

## 0.5.1

- fix the callback immutable type paramaters

- fix the callback lifetime parameter

## 0.5.0

- improve the api of Context

## 0.4.3

- clean the error

- clean Ustr::from

- clean the module import

- update rust version to 2021

- improve the export of structs

- some documents update 

## 0.4.2

- fix the display of alias hint

- using TryFrom instead of From when convert CreateInfo to *Opt

## 0.4.1

- add example index constituent spyder

- fix the option index backward hint

- move delay parser option check after the value set

- fix duplicate alias name

- some documents update

## 0.4.0

- add with_* api for struct need build

- remove tools mod

- remove initialize_*

- make strict mode default of *Parser

- refactor ArgStream and change the Parser::parser api

- fix some bugs of Index