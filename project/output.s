.global _main
.align 4
.text
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

_main:
str x30, [sp, #-16]!
sub sp, sp, #48
mov x10, #5
str x10, [sp, #40]
mov w10, #2
strh w10, [sp, #38]
mov w10, #1
strb w10, [sp, #37]
mov x10, #30
str x10, [sp, #29]
ldr x10, [sp, #29]
str x10, [sp, #21]
mov x10, #10
str x10, [sp, #13]
mov w10, #30
strb w10, [sp, #12]
ldrb w10, [sp, #12]
strb w10, [sp, #11]
mov x10, #0
str x10, [sp, #3]
mov x10, #1
str x10, [sp, #3]
sub sp, sp, #16
ldr x10, [sp, #56]
str x10, [sp, #8]
ldr x10, [sp, #37]
str x10, [sp, #0]
bl _test
add sp, sp, #16
ldr x0, [sp, #3]
bl _term
add sp, sp, #48
ldr x30, [sp], #16
ret

_4:
str x30, [sp, #-16]!
ldr x30, [sp], #16
ret

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
str x30, [sp, #-16]!
sub sp, sp, #16
mov x10, #5
str x10, [sp, #8]
mov x10, #10
str x10, [sp, #8]
add sp, sp, #16
ldr x30, [sp], #16
ret

