import numpy as np
import matplotlib.pyplot as plt
import re

def remove_chars(s, chars):
	for char in chars:
		s = s.replace(char, '')
	return s

def load_data(filename):
	p = []
	LA = []
	entropy = []
	entropy_err = []

	with open('data.txt', 'r') as f:
		lines = [i.strip() for i in f.readlines()]
		num_lines = len(lines)
		for i in range(num_lines//3):
			p.append(float(lines[3*i]))
			LA = remove_chars(lines[3*i + 1], [']','[',',']).split(' ')
			d = [remove_chars(s, ['(',')']) for s in re.findall('\[[^\]]*\]|\([^\)]*\)|\"[^\"]*\"|\S+', remove_chars(lines[3*i + 2], [']', '[',',']))]
			entropy.append([float(i.split(' ')[0]) for i in d])
			entropy_err.append([float(i.split(' ')[1]) for i in d])

	LA = [int(i) for i in LA]
	return p, LA, entropy, entropy_err

def plot_data(ax, p, LA, entropy, entropy_err):
	for n,prob in enumerate(p):
		ax.errorbar(LA, entropy[n], entropy_err[n], label=f'p = {prob}')
	ax.set_xlabel(r'$L_A$', fontsize=15)
	ax.set_ylabel(r'$\bar{S^2_A}$', fontsize=15)
	ax.legend()

fig, ax = plt.subplots()
plot_data(ax, *load_data("data.txt"))

plt.show()

