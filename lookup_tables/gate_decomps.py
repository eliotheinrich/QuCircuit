import numpy as np
from clifford_group import *

V_str = 'h s h s'
W_str = 'h s'
A = ['I', V_str, W_str, 'h', 'h ' + V_str, 'h ' + W_str]
B = ['I', 'h s s h', 'h s s h s s', 's s']
C = []


for a in A:
	for b in B:
		C.append(f'{a} {b}'.replace('h h', '').replace('I', '').strip()[::-1])


def decomp_to_matrix(s):
	M = np.eye(2)
	for g in s.split(' '):
		if g == 'h':
			M = H @ M
		elif g == 's':
			M = S @ M
	return M

for i in range(NUM_CLIFFORDS):
	assert(arr_prop(decomp_to_matrix(C[i]), C1[i]))

print(C[5])

for i in range(NUM_CLIFFORDS):
	gates = ', '.join([f'"{g}".to_string()' for g in C[i].split(' ')])
	print(f"\tgate_decomps.push(vec![{gates}]);")

print(' --- CLIFFORD CONJUGATION TABLE --- ')
def conj(U, P):
	return np.conjugate(U.T) @ P @ U

paulis = [X, Y, Z, -X, -Y, -Z]
conj_table = [-1]*NUM_CLIFFORDS
for i,c in enumerate(C1):
	for j,p in enumerate(paulis):
		if np.allclose(conj(c, Z), p):
			conj_table[i] = j


print('[', end='')
for n,c in enumerate(conj_table):
	print(c + 1, end='')
	if n != NUM_CLIFFORDS - 1:
		print(', ', end='')
print(']')