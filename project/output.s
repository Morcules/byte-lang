.global _main
.align 16
.text
_3:
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

_2:
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
mov w10, #104
strb w10, [sp, #14]
mov w10, #101
strb w10, [sp, #15]
mov w10, #108
strb w10, [sp, #16]
mov w10, #108
strb w10, [sp, #17]
mov w10, #111
strb w10, [sp, #18]
mov w10, #32
strb w10, [sp, #19]
mov w10, #119
strb w10, [sp, #20]
mov w10, #111
strb w10, [sp, #21]
mov w10, #114
strb w10, [sp, #22]
mov w10, #108
strb w10, [sp, #23]
mov w10, #100
strb w10, [sp, #24]
mov w10, #32
strb w10, [sp, #25]
ldrb w10, [sp, #16]
strb w10, [sp, #35]
ldr x10, [sp, #36]
ldr x11, [sp, #44]
cmp x10, x11
b.ne _2
b.eq _3
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

_term:
str x30, [sp, #-16]!
mov x16, #1
svc #0x80
ldr x30, [sp], #16
ret

