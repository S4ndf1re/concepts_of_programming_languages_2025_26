# bound = 100000000
# print(bound)
# sum = 0
# for i in range(bound):
#     sum += i;


def add(a, b):
    return a + b

sum = 0
for i in range(100):
    sum = add(sum, i)

print(add("foo", "bar"))