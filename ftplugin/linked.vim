if exists("g:loaded_linked")
    finish
endif
let g:loaded_linked = 1

let s:path = resolve(expand('<sfile>:p:h') . "/../")
let s:inst = libcallex#load(s:path . "/target/release/liblinked.so")

function! StartsWith(longer, shorter) abort
  return a:longer[0:len(a:shorter)-1] ==# a:shorter
endfunction

function! PrintError(msg) abort
    execute 'normal! \<Esc>'
    echohl ErrorMsg
    echomsg a:msg
    echohl None
endfunction

function! s:TextChanged()
    let current_buf = join(getline(1,'$'), "\n") 
    let ret = s:inst.call("update_content", [current_buf], "string")

    if StartsWith(ret, "Error:")
        call PrintError(ret)
    endif
endfunction

function! s:GoTo(mode_set)
    let args = {'mode': a:mode_set, 'cursor': getpos(".")}
    let ret = s:inst.call("go_to", [json_encode(args)], "string")

    if StartsWith(ret, "Link error:")
        call PrintError(ret)
    elseif !empty(ret)
        let ret = json_decode(ret)
	if has_key(l:ret, 'path')
		if l:ret['path'] =~ ".pdf$"
			let command = "evince \"" . l:ret['path'] . "\""
			if has_key(l:ret, 'text')
				let command .= ' -l "' . l:ret['text'] . '"'
			endif
			let command .= ' > /dev/null  &'

			"echohl DiagnosticInfo
			"echo command
			"echohl None

			call system(command)
		elseif l:ret['path'] =~ ".md$"
			if has_key(l:ret, 'text')
				execute 'edit' l:ret['path'] '+/' l:ret['text']
			elseif has_key(l:ret, 'line')
				execute 'edit' l:ret['path'] '+:' l:ret['line']
			else
				execute 'edit' l:ret['path']
			endif
		endif
	elseif has_key(ret, 'line')
		normal! m`
		call setpos('.', [0, ret['line'], 1, 1])
	endif
    endif
endfunction


autocmd VimEnter,BufEnter,TextChanged,InsertLeave * call <SID>TextChanged()

:nmap gf :call <SID>GoTo("Forward")<CR>
:nmap gb :call <SID>GoTo("Backward")<CR>
:nmap gF :call <SID>GoTo("ForwardEnd")<CR>
:nmap gB :call <SID>GoTo("BackwardEnd")<CR>

" create new note
function! s:add_zettel()
    let pos = search("^# ", "W")
    if pos == 0
        execute "normal Go"
    else
        execute "normal O"
    endif
    execute "r!tr -dc A-Za-z0-9 </dev/urandom | head -c 12 ; echo ' - '" | normal I#
    execute "startinsert!"
endfunction

noremap zn :call <SID>add_zettel()<CR>

" indent note
function! s:indent_zettel()
	let pos = getpos('.')
	let @/ = '#\{1,6}'
	execute "normal /\<cr>NI#\<esc>"
	call setpos('.', pos)
endfunction

function! s:undent_zettel()
	let pos = getpos('.')
	let @/ = '#\{1,6}'
	execute "normal /\<cr>Nx"
	call setpos('.', pos)
endfunction

noremap z> :call <SID>indent_zettel()<CR>
noremap z< :call <SID>undent_zettel()<CR>

" enable visible text border and do not wrap to play nicely with
" concealed URL elements
set textwidth=80
set colorcolumn=-1
set nowrap
