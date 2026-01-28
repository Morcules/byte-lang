.global _main
.align 16
.text
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

_3:
ldr x10, [sp, #72]
str x10, [sp, #6]
mov x10, #10
str x10, [sp, #6]
mov x0, #0
bl _term
b _main_1

_main:
str x30, [sp, #-16]!
sub sp, sp, #80
mov x10, #5
str x10, [sp, #72]
mov w10, #2
strh w10, [sp, #70]
mov w10, #1
strb w10, [sp, #69]
mov w10, #97
strb w10, [sp, #68]
mov x10, #30
str x10, [sp, #60]
ldr x10, [sp, #60]
str x10, [sp, #52]
mov x10, #5
str x10, [sp, #44]
mov x10, #10
str x10, [sp, #36]
mov w10, #30
strb w10, [sp, #35]
ldrb w10, [sp, #35]
strb w10, [sp, #34]
mov x10, #0
str x10, [sp, #26]
ldrb w10, [sp, #16]
strb w10, [sp, #35]
ldr x10, [sp, #36]
ldr x11, [sp, #44]
cmp x10, x11
b.ne _3
b.eq _4
b _main_1
_main_1:
mov x10, #1
str x10, [sp, #26]
sub sp, sp, #16
ldr x10, [sp, #88]
str x10, [sp, #8]
ldr x10, [sp, #68]
str x10, [sp, #0]
bl _test
add sp, sp, #16
ldr x0, [sp, #26]
bl _term
add sp, sp, #80
ldr x30, [sp], #16
ret

_4:
mov w10, #1
strb w10, [sp, #10]
mov w10, #2
strb w10, [sp, #11]
mov w10, #3
strb w10, [sp, #12]
ldrb w10, [sp, #68]
strb w10, [sp, #13]
ldrsb x14, [sp, #69]
mov x15, #0
add x11, sp, #10
strb w15, [x11, x14]
mov x0, #1
bl _term
b _main_1

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

