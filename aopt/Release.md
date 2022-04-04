
# Relase note

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