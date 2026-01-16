.global _main
.align 16
.text
_3:
ldr x10, [sp, #104]
str x10, [sp, #38]
mov x10, #10
str x10, [sp, #38]
mov x0, #0
bl _term
b _main_1

_main:
str x30, [sp, #-16]!
sub sp, sp, #112
mov x10, #5
str x10, [sp, #104]
mov w10, #2
strh w10, [sp, #102]
mov w10, #1
strb w10, [sp, #101]
mov w10, #97
strb w10, [sp, #100]
mov x10, #30
str x10, [sp, #92]
ldr x10, [sp, #92]
str x10, [sp, #84]
mov x10, #5
str x10, [sp, #76]
mov x10, #10
str x10, [sp, #68]
mov w10, #30
strb w10, [sp, #67]
ldrb w10, [sp, #67]
strb w10, [sp, #66]
mov x10, #0
str x10, [sp, #58]
ldr x10, [sp, #68]
ldr x11, [sp, #76]
cmp x10, x11
b.ne _3
b.eq _4
b _main_1
_main_1:
mov x10, #1
str x10, [sp, #58]
sub sp, sp, #16
ldr x10, [sp, #120]
str x10, [sp, #8]
ldr x10, [sp, #100]
str x10, [sp, #0]
bl _test
add sp, sp, #16
ldr x0, [sp, #58]
bl _term
add sp, sp, #112
ldr x30, [sp], #16
ret

_term:
str x30, [sp, #-16]!
sub sp, sp, #16
mov w10, #10
str w10, [sp, #12]
mov x1, #12
mov x16, #1
svc #0x80
add sp, sp, #16
ldr x30, [sp], #16
ret

_4:
mov x10, #1
str x10, [sp, #14]
mov x10, #2
str x10, [sp, #22]
mov x10, #3
str x10, [sp, #30]
mov x10, #4
str x10, [sp, #38]
mov x0, #1
bl _term
b _main_1

_test:
str x30, [sp, #-16]!
sub sp, sp, #16
mov x10, #15
str x10, [sp, #40]
ldr x10, [sp, #40]
str x10, [sp, #8]
add sp, sp, #16
ldr x30, [sp], #16
ret

