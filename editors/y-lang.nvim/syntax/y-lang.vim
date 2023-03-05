syn keyword whyKeyword let mut declare import 

syn keyword whyConditional if else 

syn match whyOperator display "\%(+\|-\|/\|*\|=\|:=\|\^\|&\||\|!\|>\|<\|%\)=\?"

syn match whyIdentifier '"([a-zA-z0-9]|_)*"'

syn match whyLineComment "//.*$"
syn region whyBlockComment start=+/\*+ end=+\*/+

syn match whyDecimalNumber '\d\+'
syn match whyHexNumber '0x\d\+'

hi def link whyDecimalNumber whyNumber
hi def link whyHexNumber whyNumber

syn region whyStrings start=+"+ end=+"+ end=+$+ 

syn keyword whyBoolean true false

syn keyword	whyType any int str bool void unknown

syn match whyFuncCall "\w\(\w\)*("he=e-1,me=e-1

syn region    rustAttribute   start="#\[" end="\]" 

hi def link whyKeyword Keyword 
hi def link whyConditional Conditional
hi def link whyOperator Operator
hi def link whyIdentifier Identifier
hi def link whyLineComment Comment
hi def link whyBlockComment Comment
hi def link whyNumber Number
hi def link whyStrings String
hi def link whyType Type
hi def link whyBoolean Boolean
hi def link whyFuncCall Function
hi def link rustAttribute PreProc
