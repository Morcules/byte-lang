.global _main
.align 16
.text
_3:
ldr x10, [sp, #56]
str x10, [sp, #3]
mov x10, #10
str x10, [sp, #3]
mov x0, #0
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

_test:
str x30, [sp, #-16]!
sub sp, sp, #16
mov x10, #15
str x10, [sp, #8]
ldr x10, [sp, #8]
str x10, [sp, #8]
add sp, sp, #16
ldr x30, [sp], #16
ret

_main:
str x30, [sp, #-16]!
sub sp, sp, #64
mov x10, #5
str x10, [sp, #56]
mov w10, #2
strh w10, [sp, #54]
mov w10, #1
strb w10, [sp, #53]
mov x10, #30
str x10, [sp, #45]
ldr x10, [sp, #45]
str x10, [sp, #37]
mov x10, #5
str x10, [sp, #29]
mov x10, #10
str x10, [sp, #21]
mov w10, #30
strb w10, [sp, #20]
ldrb w10, [sp, #20]
strb w10, [sp, #19]
mov x10, #0
str x10, [sp, #11]
ldr x10, [sp, #21]
ldr x11, [sp, #29]
cmp x10, x11
b.ne _3
b.eq _4
b _main_1
_main_1:
mov x10, #1
str x10, [sp, #11]
sub sp, sp, #16
ldr x10, [sp, #72]
str x10, [sp, #8]
ldr x10, [sp, #53]
str x10, [sp, #0]
bl _test
add sp, sp, #16
ldr x0, [sp, #11]
bl _term
add sp, sp, #64
ldr x30, [sp], #16
ret

_4:
mov x0, #1
bl _term
b _main_1

