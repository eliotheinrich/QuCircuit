from clifford_group import *
import numpy as np

print(' --- Making 2-state lookup table --- ')
CZ = np.diag([1,1,1,-1])

S20 = [[None for _ in range(NUM_CLIFFORDS)] for _ in range(NUM_CLIFFORDS)]
S21 = [[None for _ in range(NUM_CLIFFORDS)] for _ in range(NUM_CLIFFORDS)]
HH = np.kron(H, H)
p = np.array([1., 0., 0., 0.])

def graph_state(c1, c2, joined):
	if joined:
		return np.kron(C1[c1], C1[c2]) @ CZ @ HH @ p
	else:
		return np.kron(C1[c1], C1[c2]) @ HH @ p

for c1 in range(NUM_CLIFFORDS):
	for c2 in range(NUM_CLIFFORDS):
		S20[c1][c2] = graph_state(c1, c2, False)
		S21[c1][c2] = graph_state(c1, c2, True)

take_data = False
if take_data:
	CZ_table0 = [[[] for _ in range(NUM_CLIFFORDS)] for _ in range(NUM_CLIFFORDS)]
	CZ_table1 = [[[] for _ in range(NUM_CLIFFORDS)] for _ in range(NUM_CLIFFORDS)]
	for c1 in range(NUM_CLIFFORDS):
		print(c1)
		for c2 in range(NUM_CLIFFORDS):
			result0 = CZ @ S20[c1][c2]
			result1 = CZ @ S21[c1][c2]
			for a1 in range(NUM_CLIFFORDS):
				for a2 in range(NUM_CLIFFORDS):

					if arr_prop(result0, S20[a1][a2]):
						CZ_table0[c1][c2].append((0, a1, a2))
					elif arr_prop(result0, S21[a1][a2]):
						CZ_table0[c1][c2].append((1, a1, a2))

					if arr_prop(result1, S20[a1][a2]):
						CZ_table1[c1][c2].append((0, a1, a2))
					elif arr_prop(result1, S21[a1][a2]):
						CZ_table1[c1][c2].append((1, a1, a2))

	with open('data0.bin', 'wb') as f:
		pkl.dump(CZ_table0, f)
	with open('data1.bin', 'wb') as f:
		pkl.dump(CZ_table1, f)
else:
	with open("data0.bin", "rb") as f:
		CZ_table0 = pkl.load(f)
	with open("data1.bin", "rb") as f:
		CZ_table1 = pkl.load(f)

CZ_table = []
def prune_options(c1, c2, options):
	ZGATE1 = c1 in ZGATES
	ZGATE2 = c2 in ZGATES

	for option in options:
		if ZGATE1 and ZGATE2:
			if option[1] in ZGATES and option[2] in ZGATES:
				return option
		elif ZGATE1:
			if option[1] in ZGATES:
				return option
		elif ZGATE2:
			if option[2] in ZGATES:
				return option
		else:
			return option

CZ_table = [[[(0,0,0), (0,0,0)] for _ in range(NUM_CLIFFORDS)] for _ in range(NUM_CLIFFORDS)]
for c1 in range(NUM_CLIFFORDS):
	for c2 in range(NUM_CLIFFORDS):
		CZ_table[c1][c2][0] = prune_options(c1, c2, CZ_table0[c1][c2])
		CZ_table[c1][c2][1] = prune_options(c1, c2, CZ_table1[c1][c2])

# Checking
lens = []
check_valid = True
if check_valid:
	valid = True
	for c1 in range(NUM_CLIFFORDS):
		for c2 in range(NUM_CLIFFORDS):
			for linked in [0,1]:
				option = CZ_table[c1][c2][linked]

				s1 = graph_state(c1, c2, linked)
				s2 = graph_state(option[1], option[2], option[0])

				if not arr_prop(CZ @ s1, s2):
					valid = False
				if c1 in ZGATES and option[1] not in ZGATES:
					valid = False
				if c2 in ZGATES and option[2] not in ZGATES:
					valid = False
				
			
	print(f'Table valid: {valid}')

print('[', end='')
i = 0
printed_newline = False
for n,row in enumerate(CZ_table):
	print('[', end='')
	for m,items in enumerate(row):
		i += 1
		print(f'[({"true" if items[0][0] else "false"}, {items[0][1]}, {items[0][2]}), ', end='')
		print(f'({"true" if items[1][0] else "false"}, {items[1][1]}, {items[1][2]})]', end=f", " if m != NUM_CLIFFORDS - 1 else "")
		if i%4 == 0 and m != NUM_CLIFFORDS - 1:
			print("")
	print("]", end=",\n " if n != NUM_CLIFFORDS - 1 else "")

print("]")

print(CZ_table[11][16][0])

