# Hand decompiled day 19 part 2

a,b,c,d,e,f = 1,0,0,0,0,0

c += 2
c *= c
c *= 209
b += 2
b *= 22
b += 7
c += b
if a == 1:
    b = 27
    b *= 28
    b += 29
    b *= 30
    b *= 14
    b *= 32
    c += b
    a = 0

for d in range(1, c+1):
    if c % d == 0:
        a += d
print(a)