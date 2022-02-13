highlight zettelTag guifg=#c9be51
highlight zettelId guifg=grey
highlight zettelTitle gui=bold cterm=bold guifg=#ef5939
highlight markdownLinkText guifg=blue
highlight mkdURL ctermbg=blue guifg=grey

syn match zettelTag "§\S\+"

syn region mkdID matchgroup=mkdDelimiter    start="\["    end="\]" contained oneline conceal
syn region mkdURL matchgroup=mkdDelimiter start="(" end=")" contained oneline conceal
syn region markdownLinkText matchgroup=markdownLinkTextDelimiter
    \ start="!\=\[\%(\_[^]]*]\%( \=[[(]\)\)\@=" end="\]\%( \=[[(]\)\@="
    \ nextgroup=mkdURL,mkdID skipwhite
    \ contains=@markdownInline,markdownLineStart
    \ concealends

syn match zettelId "\w\{12}"
syn region zettelTitle matchgroup=mkdHeading
            \ start="- " end="$"
            \ contained oneline

syn region zettelHeader matchgroup=mkdHeading 
            \ start="^#\{1,6} " end="$"
            \ contains=zettelId,zettelTitle

" This gets rid of the nasty _ italic bug in tpope's vim-markdown
" block $$...$$
syn region math start=/\$\$/ end=/\$\$/
" inline math
syn match math '\$[^$].\{-}\$'

" actually highlight the region we defined as "math"
hi link math Statement
