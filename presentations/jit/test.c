#include <stdio.h>
#include <stdlib.h>
#include <sys/mman.h>


typedef union {
    char* values;
    float valuef;
} InterpreterValue;


void set_value(int value_id, InterpreterValue value) {
    printf("THis is a callback");
}


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
    program[5] = // make function call &set_value
    program[6] = // make gc callback
    program[7] = 0xC3;

    fnptr = (int (*)(void))program;
    a = fnptr();


    void (*fn_addr)(int, InterpreterValue) = &set_value;

    printf("Result = %X\n", a);
    return 0;
}
