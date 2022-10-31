import numpy as np
from clifford_group import *

print('\n --- MATRIX DECOMP --- ')
decomps = {'H': 'X Z Z Z X', 
		   'X': 'X', 'Z': 'Z',
		   'X2': 'X X', 'Z2': 'Z Z',
		   'S': 'Z Z Z', 'W': 'H S', 'V': 'W W', 'I':'' , '': ''}

simplification_rules = { decomps['H'] + ' ' + decomps['H']: '',
						'  ': ' ',
						'Z Z Z Z': '',
						'X X X X': '',

						'Z X X Z': 'X X',
						'X Z Z X': 'Z Z',

						'Z X Z X Z': 'X X X',
						'X Z X Z X': 'Z Z Z',

						'Z X X X Z Z Z X': 'X Z',
						'X Z Z Z X X X Z': 'Z X',

						'Z X X X Z Z': 'Z Z Z X',
						'X Z Z Z X X': 'X X X Z',
						'Z Z X X X Z': 'X Z Z Z',
						'X X Z Z Z X': 'Z X X X',
						'Z Z X Z Z Z': 'X X X Z',
						'X X Z X X X': 'Z Z Z X',
						'X Z Z Z X Z': 'Z Z Z X',
						'Z X X X Z X': 'X X X Z',
						'X Z X X X Z': 'Z X X X',
						'Z X Z Z Z X': 'X Z Z Z',
						
						'X Z X Z Z Z': 'Z X',
						'Z X Z X X X': 'X Z',

						'Z X X X Z X X': 'X X X Z X',
						'X Z Z Z X Z Z': 'Z Z Z X Z',

						'X X X Z X Z': 'Z X',
						'Z Z Z X Z X': 'X Z',

						'X Z Z Z X Z Z Z': 'Z X X X',
						'Z X X X Z X X X': 'X Z Z Z'

}
mat_dict = {}

class Sequence:
	def __init__(self, seq):
		self.title = seq
		self.decomposed = Sequence.decompose(seq)

	def decompose(s):
		d = s
		changed = True
		while changed:
			changed = False
			items = d.split(' ')
			d = ''
			for item in items:
				new_item = decomps[item]
				if new_item != item:
					changed = True
				d += new_item + ' '

		# Now simplify
		changed = True
		while changed:
			changed = False
			for before,after in simplification_rules.items():
				d_old = d
				d = d.replace(before, after)
				if d_old != d:
					changed = True
		
		items = d.split(' ')
		items = [item for item in items if item != '']
		d = ' '.join(items)

		if d == '':
			d = 'I'

		return d

	def is_valid(self):
		items = self.title.split(' ')
		M = np.identity(2)
		for item in items:
			M = M @ operators[item]

		items = self.decomposed.split(' ')
		Md = np.identity(2)
		for item in items:
			if item == 'X':
				Md = Md @ sqrtX
			elif item == 'Z':
				Md = Md @ np.conjugate(S)
		
		return arr_prop(M, Md)
	
	def to_rust_arr(self):
		s = []
		items = self.decomposed.split(' ')
		for i in range(5 - len(items)):
			s.append('IDGATE')
		for item in items:
			if item == 'I':
				s.append('IDGATE')
			elif item == 'X':
				s.append('SQRTXGATE')
			else:
				s.append('SQRTZGATE')
		return s[::-1]

	def __str__(self):
		return f'{self.title}: {self.decomposed}, valid: {self.is_valid()}'

A_str = ['I', 'V', 'W', 'H', 'H V', 'H W']
B_str = ['I', 'X2', 'X2 Z2', 'Z2']
sequences = []
for a in A_str:
	for b in B_str:
		sequences.append(Sequence(a + ' ' + b))

max_L = 0
for seq in sequences:
	L = len(seq.decomposed.split(' '))
	if L > max_L:
		max_L = L
	print(seq)
print(f'MAX L: {max_L}')

rust_arr = []
for i in range(len(sequences)):
	rust_arr.append(sequences[i].to_rust_arr())

print('[', end='')
for m,row in enumerate(rust_arr):
	print('[', end='')
	for n,gate in enumerate(row):
		print(f'{gate}', end=", " if n != 4 else "")
	print(']', end=',\n' if m != NUM_CLIFFORDS - 1 else "")
print(']')
