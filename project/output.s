.global _main
.align 4
.text
_test:
str x30, [sp, #-16]!
sub sp, sp, #16
mov w10, #15
str w10, [sp, #44]
ldrsw x10, [sp, #44]
str w10, [sp, #12]
add sp, sp, #16
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
_main:
str x30, [sp, #-16]!
sub sp, sp, #48
mov w10, #5
str w10, [sp, #44]
mov w10, #2
strh w10, [sp, #42]
mov w10, #1
strb w10, [sp, #41]
mov x10, #30
str x10, [sp, #33]
ldr x10, [sp, #33]
str x10, [sp, #25]
mov x10, #10
str x10, [sp, #17]
mov w10, #30
strb w10, [sp, #16]
ldrb w10, [sp, #16]
strb w10, [sp, #15]
mov x10, #0
str x10, [sp, #7]
mov x10, #1
str x10, [sp, #7]
sub sp, sp, #16
ldrsw x10, [sp, #60]
str w10, [sp, #12]
ldr x10, [sp, #41]
str x10, [sp, #4]
bl _test
add sp, sp, #16
ldr x0, [sp, #7]
bl _term
add sp, sp, #48
ldr x30, [sp], #16
ret
