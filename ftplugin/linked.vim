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
    call s:inst.call("update_content", [current_buf], "")
    mode
endfunction

function! s:GoTo(mode_set)
    let args = {'mode': a:mode_set, 'cursor': getpos(".")}
    let ret = s:inst.call("go_to", [json_encode(args)], "string")

    if StartsWith(ret, "Link error:")
        call PrintError(ret)
    elseif !empty(ret)
        let ret = json_decode(ret)
        :echo ret
        call setpos('.', [0, ret['line'], 1, 1])
    endif
endfunction


autocmd VimEnter,TextChanged,InsertLeave * call <SID>TextChanged()

:nmap gf :call <SID>GoTo("fort")<CR>
:nmap gb :call <SID>GoTo("back")<CR>
:nmap gF :call <SID>GoTo("fortend")<CR>
:nmap gB :call <SID>GoTo("backend")<CR>
