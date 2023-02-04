default rel

section .data
    message db "Hello, World!", 0   ; null-terminated string
    str: times 64 db 0

section .text
    global _main

int_to_str:
        push    rbp
        mov     rbp, rsp
        lea     r8, str
        mov     dword [rbp-36], edi
        mov     dword [rbp-4], 0
        mov     dword [rbp-8], 0
        cmp     dword [rbp-36], 0
        jne     .L2
        mov     eax, dword [rbp-4]
        lea     edx, [rax+1]
        mov     dword [rbp-4], edx
        cdqe
        mov     byte [r8 + rax], 48
        mov     eax, dword [rbp-4]
        cdqe
        mov     byte [r8 + rax], 0
        jmp     .L1
.L2:
        cmp     dword [rbp-36], 0
        jns     .L5
        mov     dword [rbp-8], 1
        neg     dword [rbp-36]
        jmp     .L5
.L8:
        mov     edx, dword [rbp-36]
        movsx   rax, edx
        imul    rax, rax, 1717986919
        shr     rax, 32
        mov     ecx, eax
        sar     ecx, 2
        mov     eax, edx
        sar     eax, 31
        sub     ecx, eax
        mov     eax, ecx
        sal     eax, 2
        add     eax, ecx
        add     eax, eax
        sub     edx, eax
        mov     dword [rbp-24], edx
        cmp     dword [rbp-24], 9
        jle     .L6
        mov     eax, dword [rbp-24]
        add     eax, 87
        mov     ecx, eax
        jmp     .L7
.L6:
        mov     eax, dword [rbp-24]
        add     eax, 48
        mov     ecx, eax
.L7:
        mov     eax, dword [rbp-4]
        lea     edx, [rax+1]
        mov     dword [rbp-4], edx
        cdqe
        mov     byte [r8 + rax], cl
        mov     eax, dword [rbp-36]
        movsx   rdx, eax
        imul    rdx, rdx, 1717986919
        shr     rdx, 32
        mov     ecx, edx
        sar     ecx, 2
        cdq
        mov     eax, ecx
        sub     eax, edx
        mov     dword [rbp-36], eax
.L5:
        cmp     dword [rbp-36], 0
        jne     .L8
        cmp     dword [rbp-8], 0
        je      .L9
        mov     eax, dword [rbp-4]
        lea     edx, [rax+1]
        mov     dword [rbp-4], edx
        cdqe
        mov     byte [r8 + rax], 45
.L9:
        mov     eax, dword [rbp-4]
        cdqe
        mov     byte [r8 + rax], 0
        mov     dword [rbp-12], 0
        mov     eax, dword [rbp-4]
        sub     eax, 1
        mov     dword [rbp-16], eax
        jmp     .L10
.L11:
        mov     eax, dword [rbp-12]
        cdqe
        movzx   eax, byte [r8 + rax]
        mov     byte [rbp-17], al
        mov     eax, dword [rbp-16]
        cdqe
        movzx   edx, byte [r8 + rax]
        mov     eax, dword [rbp-12]
        cdqe
        mov     byte [r8 + rax], dl
        mov     eax, dword [rbp-16]
        cdqe
        movzx   edx, byte [rbp-17]
        mov     byte [r8 + rax], dl
        add     dword [rbp-12], 1
        sub     dword [rbp-16], 1
.L10:
        mov     eax, dword [rbp-12]
        cmp     eax, dword [rbp-16]
        jl      .L11
.L1:
        pop     rbp
        ret

print:
    mov     rdi, 1 ; stdout
    mov     rax, 0x2000004 ; write
    syscall
    ret

_main:
    mov rdi, 10001289 
    call int_to_str
    lea     rsi,  str   ; memory address of the string
    mov     rdx, 13         ; number of bytes to write
    call    print

exit:
    mov     rax, 0x2000001 ; exit
    mov     rdi, 0 
    syscall
