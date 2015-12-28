import itertools

def eqn(a):
    return a[0] + a[1] * a[2] * a[2] + a[3] * a[3] * a[3] - a[4]

arr = [2, 3, 5, 7, 9]

for perm in itertools.permutations(arr):
    if eqn(perm) == 399:
        print perm
