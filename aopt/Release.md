
# Relase note

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