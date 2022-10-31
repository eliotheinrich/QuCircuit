from clifford_group import *

table = np.zeros((NUM_CLIFFORDS, NUM_CLIFFORDS), dtype=int)

# Computing local Clifford multiplication table
for i in range(NUM_CLIFFORDS):
	for j in range(NUM_CLIFFORDS):
		for k in range(NUM_CLIFFORDS):
			if arr_prop(C1[i] @ C1[j], C1[k]):
				table[i][j] = k
				break

print(' --- CLIFFORD TABLE --- ')
print(np.array2string(table, separator=', '))

# Identifying interesting matrices

print('\n ---  INDEXING MATRICES --- ')
for name, m in operators.items():
	for n,c in enumerate(C1):
		if arr_prop(c, m):
			print(f'{name}: {n}')


print(fix_gauge(sqrtX))

