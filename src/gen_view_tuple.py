# Generates macro invocations in viewtuple.rs

max_elements = 128

print("pub const VIEW_TUPLE_MAX_ELEMENTS: usize = %d;" % max_elements)

for i in range(1, max_elements+1):
    # impl_view_tuple!(3; V0, V1, V2; 0, 1, 2);
    print("impl_view_tuple!(%d; " % i, end="")
    for j in range(i-1):
        print("V%d, " % j, end="")
    print("V%d; " % (i-1), end="")
    for j in range(i-1):
        print("%d, " % j, end="")
    print("%d; " % (i-1), end="")
    for j in reversed(range(1, i)):
        print("%d, " % j, end="")
    print("0);")
