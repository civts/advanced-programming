pub const BANNER: & [&str]= 
&[
"                                         This is the SOL market file                                ",
"",
"                                             (0[)C(..1CUCz~                                         ",
"                                              nY__xQ  1v-]zY                                        ",
"                     .}xI                     .*-__}h' *---xp                                       ",
"                     ~U{vCc,                   o?-__/z *?-__cx                                      ",
"                      `rZ]+zC_                 *?-__-n~o?-__}O:                                     ",
"                        !q}+-Yt.               *--___unm?--_?J{                                     ",
"                         /v___ct              [Z----_u@/?---_u)             '];                     ",
"                         )X-__+#i            .C(-----ca{----_uc'            ;h0,                    ",
"             lvc]        )z-___zb            {Y]???-QBz?----_td,            ;UjC       ~xt/rx.      ",
"             .U|fr`      )v-___[*           ~m}???n8YJQ]?----?q;            w[+#'   .)U)+|zr,       ",
"              id?|n`     )v____?#\"         {q]??]Ca1-}w0???---Z+          `Oj_+af  +Of+_Q[          ",
"               /Q-{Uj!.  )v_____od~      'tC????uo/?--_X8v[??-Yh        `tY[__+oY'tX-_?w\"           ",
"                nc--]jJQo@b]_____/JOohZCUJ/??---Wt??----)Mz]?--M~   ._rOJ)--___M<c/---J:            ",
"                'vv]]]]]]}vbdc}____-----?????-?-????????-tj]]??|BooqCX1?-----_z%C[?--f}             ",
"                 .)Q([[[[[]]}vZbX----?????]fJJQwkhhhbwLCJj[]]]?????????-----?coz??--?f!      in>    ",
"                   `Ybt[[[[[[]]}mm]???|U0qm0vjjrjjjjfffjUmp&mz{]]]????--{zJOMZ|]??--J)      |O{c    ",
"                     ?k/[[[[[[]]]||UmqQxjjjjjuU0mmmmZQYxfjjrnZ*dX]]]}Z8aCv}--????(C%Z     ]mc-Ul    ",
"                      +M{[[[[[[]jwdcrrrUwb&kYuvj)]??}|nvcwowCrjrZhO{)m[????--)UdpU|?rCZmQJ|--x{     ",
"                       ro[[[[[u*Oxrrcdbz}\"^^^^^^^^^^^^^^^^^>cpbUjjCMZ/?????]Oou[????------_{Y-      ",
"I1.                    iW}[[nqwnrrJ*L}^^^^^^^^^^^^^^^^^^^^^^^^<noQnrCbC[??]MQ}[]]]????--]un}        ",
">zv`                  ,hL?[mpvxrvam>^^^^^^^^^^^^^^^^^^^^^^^^^^^^,|bmxn0Mv?vQ[[[[[[]?[cQCjl          ",
">xr0[               ;zO|?/kJxxnwM{^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^,x8JrukLx{[[[}[[uan?.             ",
":L[?rCzx}\"     .]nXCX}??toXxrX*x^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^IZwxxdk1[[}}1O/.                ",
" ]u????]rUCLZmQCn]?????/&cjjXp+^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^!wqjrdo[[[[p?                  ",
"  {J??????????????????1WvrjQ#!^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^;#Qffbh[]d1                   ",
"   ;0v[]???]]]]]????--OOrrU&,^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^>&ntfdC]M'                   ",
"     ~YLJj[]]]][jYY}-|hujuok^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^upftxw)08ul                 ",
"        ^{rxvXvx}l]cmCmftC@@@@@@@@@@@@@@%Oc{^^^^^^^^^^^^^^^^^^^^^^^^^^^^i#cttQc/)uLbn_  ._xrxvunjI  ",
"                 ^?rC@Ltjd8@@@@&#oaaao#MW%@@BZu-^^^^i(uvYwhoobZXuu1,^^^^:YQt/Jc1&j?-{CLJCx]?----]JY^",
"             `1CLUx/)fxtxd(M@B*hhhhhhhhhhho%@@@@@@@@@@@B&WMMW&%@@@@@BZv}^/pj/Yv?uoz]????????-----_|q",
"           .vqt[[[[[|bftx0,*@&kkhhhhhhhhhhkk@@@@@@@@B*kkkkkkhkkkh#%@@@@@BkdxtXU[?(OWCcxYCJ1?]rOq0JJz",
"}wZx     `|0|[fYJv)}|dt/xC^*@Wkkhhhhhhhhhhhk@@@Cvm@@8khhhhhhhhhhhkhB@@@@8LhntzL}]???XCCv]-[Uz].    ]",
"\"c)|JrxxcCf[/LLLLOMZ|df/xC\"Z@&hkhhhhhhhhhhkW@h_^^^YB%khhhhhhhhhhhkk&@@@@ar#ntX0{]]???---}Uv^        ",
" IYf[[[}}}}}}}}[[[[C*qctr0~l@@Whkhhhhhhhhk#@o>^^^^,*Bokkhhhhhhhhhkk8@%u,\"jqjtJO}]]???1XZnl          ",
"   )CJt}}}}}}{{11}[]Y@Ctt0|^x8@BM*kkkkhk*&@J^^^^^^^?@8hhhhhhhhhhkkhB@L\"^l#QftCUq8JzvxtI             ",
"     ^{xncXXznxxxzZ/[Cwj/UQ;^^_nm@@@@@@@%C_^^^^^^^^^c@8ohkhhhhhhka%@#;^^-*vttQv_Xdx,.<jUqdpwX|,     ",
"                  ;oX/hntjki^^^^^]dzi,\"^^^^^^^^^^^^^^]h@@&MMMMMM%@@Z:^^^8mftrkx__]vJJz1------(CZ|   ",
"                   ;M]JmftJd^^^^^^)*x0X[^^^^^^^^^^^^^^^!uc0h&@@&bz_^^^^OMfttxq[---___----------_jU_ ",
"                   .W}]#zttbC^^^^^^)#i;jZ*Yu~^^^^^^^^^^^^^^^;nx:^^^^^^nMftttkc???-----)nXUUc/?-___u|",
"                   Yq]?1Wrtfbx^^^^^^[k<   <rub%oLcuuvvvvvJOu%X,^^^^^^[hntttU*????]nCXx(i^. `i|ruY1_0",
"                  )Q[]]]ranfxbX:^^^^^_hc        `>?[}]+;  -or^^^^^^^!bzfttz#}]?fLr>             '1YQ",
"                `cJ}]]]]]zkxfrpL,^^^^^IC&|\"             ^Jk?^^^^^^^,aXfftcb([]0r                   |",
"              <xZr]]]]][[[zhnfjO&/,^^^^^>0%Y|`        >x8ul^^^^^^^{avfffxdf]]{q;                    ",
"           +uLx]??]][{)1}[[uMLjjnp*u^^^^^^^tY&BbYnuCa%0r^^^^^^^^-OZjfffYMt]]?{d;                    ",
"         ;UX--_-nYxxrjh@qLm%JdhnjjnZ&v>^^^^^^^\"+/xf]\"^^^^^^^^;|omnfftfQd}]]]??mt                    ",
"        ^X|--fnj;._ruJt-??)W]]n&ZxjjrLhQxi^^^^^^^^^^^^^^^^IxJ*0rffftfQC{[]]]??(W;                   ",
"        v)-c|;irXJf?----??mw??-?QMwxjjjrJwoZvn(<:\",l-/uvC#bOjfffffcZObp{[[]]???n#:                  ",
"       ;Z(t'\"cv]-__--????|a)??---[zWqYjfffffjYOZZZZZOO0Yt/ttttfrmb01]{Z%Y][]]??-Xpl                 ",
"       if! ;L?--_---?????ch{???xCC/]j0*bZzftttt////////////vOd#0n[[[]]?)0au}]]???twv`               ",
"           w??--?fJCJLphZon]]?U%)[]]]]]}z0*%aqZZZZZZZOOq*BM0J(]]]]]]]]]]]1Zb)]????-?zc>             ",
"          ,j??)vr~      )q[]]]mL]]]]rJnxLC1[}rJCCLLLCJYx)]][][uJvzQdLr[[]]]Z%0LCCzf}-?vc            ",
"          {{]x).        :L}]]]m1??jw|   ^Qz[]]u#v]]]]rM(]]]][Xx.    ]dC}[[]]oJ   \"]rJd/(k           ",
"          j1|n          :O{]]]w{{C/'     1O{]]]}Zat]]c%([[[[(U.      ;wv[[[](J       !L1nn          ",
"          |1x           /C[]]xWwr\"       ;w|[[[[]foY]c#)[[[[X}        \"mX[[[{J_       `Z(X          ",
"           \"          `XY[[[zaX^          zm[[[[]]n8[zO{[[]]o           cO)[[tJ\"       .Y)          ",
"                     IX1]|UU|.            ,#}}}[[[]Ma8v[[[]]o            <ZQ/[101                   ",
"                     1qxr]'                LL}}}}[nx\"0j}[[]]*              'jU0Ud-                  ",
"                                           'M{}}}/r` )O1}[[[o^                 .,                   ",
"                                            CU{}}z{  'zO}[[[cY                                      ",
"                                            :w|{1vi    )oc}})m(                                     ",
"                                             (C1(n'     '{QU{(w~                                    ",
"                                             .UJ(c\"        iuLnM                                    "];
