default rel

section .data
    message db "Hello, World!", 0   ; null-terminated string

section .text
    global _main

print:
    mov     rdi, 1 ; stdout
    mov     rax, 0x2000004 ; write
    syscall
    ret

_main:
    lea     rsi, message    ; memory address of the string
    mov     rdx, 13         ; number of bytes to write
    call    print

exit:
    mov     rax, 0x2000001 ; exit
    mov     rdi, 0 
    syscall
