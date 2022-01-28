source libcallex-vim/autoload/libcallex.vim

let g:inst = libcallex#load("target/release/liblinked.so")

function! TextChanged()
    let current_buf = join(getline(1,'$'), "\n") 
    call g:inst.call("update_content", [current_buf], "")
    mode
endfunction

function! GoTo(mode, metadata)
    let ret = g:inst.call("go_to", [json_encode(metadata)], "string")
    let ret = json_decode(ret)
    :echo ret

    mode
endfunction

autocmd TextChanged,InsertLeave * call TextChanged()
nmap gf :call GoTo("forward", getpos("."))
