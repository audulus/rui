# Generates macro invocations in viewtuple.rs

for i in range(1, 128):
    # impl_view_tuple!(3; V0, V1, V2; 0, 1, 2);
    print("impl_view_tuple!(%d; " % i, end="")
    for j in range(i-1):
        print("V%d, " % j, end="")
    print("V%d" % (i-1), end="")
    print("; ", end="")
    for j in range(i-1):
        print("%d, " % j, end="")
    print("%d);" % (i-1))
