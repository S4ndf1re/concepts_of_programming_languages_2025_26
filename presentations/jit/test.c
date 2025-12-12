#include <stdio.h>
#include <stdlib.h>
#include <sys/mman.h>

int main()
{
    char *program;
    int (*fnptr)(void);
    int a;

    program = mmap(NULL, 1000, PROT_EXEC | PROT_READ | PROT_WRITE, MAP_PRIVATE | MAP_ANONYMOUS, 0, 0);

    program[0] = 0xB8;
    program[1] = 0x34;
    program[2] = 0x12;
    program[3] = 0;
    program[4] = 0;
    program[5] = 0xC3;

    fnptr = (int (*)(void))program;
    a = fnptr();

    printf("Result = %X\n", a);
    return 0;
}
