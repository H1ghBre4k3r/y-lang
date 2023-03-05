" Title:        Example Plugin
" Description:  A plugin to provide an example for creating Neovim plugins.
" Last Change:  8 November 2021
" Maintainer:   Example User <https://github.com/example-user>

" Prevents the plugin from being loaded multiple times. If the loaded
" variable exists, do nothing more. Otherwise, assign the loaded
" variable and continue running this instance of the plugin.
if exists("g:loaded_y_lang")
    finish
endif
let g:loaded_y_lang = 1

au BufRead,BufNewFile,BufWritePre *.why   setfiletype y-lang
