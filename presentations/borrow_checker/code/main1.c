#include "stdlib.h"
#include "stdio.h"

int main()
{
    int *abc = malloc(sizeof(int) * 1);
    *abc = 10;

    printf("%d", *abc);
    return 0;
}