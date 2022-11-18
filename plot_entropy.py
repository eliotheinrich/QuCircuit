from cmath import isclose
import numpy as np
import matplotlib.pyplot as plt
import json

class DataSlide:
	def __init__(self, keys, vals):
		self.data = dict(zip(keys, vals))
		
	def __getitem__(self, key):
		return self.data[key]

class DataFrame:
	def __init__(self):
		self.slides = []

	def add_dataslide(self, slide):
		self.slides.append(slide)

	def get_property_with_id(self, key, id):
		return self.slides[id][key]

	def get_property(self, key):
		val = []
		for df in self.slides:
			val.append(df[key])
		return np.array(val)
	
	def query_key(self, key, val):
		new_df = DataFrame()
		for slide in self.slides:
			if np.isclose(slide[key], val):
				new_df.add_dataslide(slide)
		return new_df

def parse_datafield(s):
	return s[list(s.keys())[0]]

def load_data(filename):
	data = DataFrame()
	with open(filename, 'r') as f:
		json_contents = json.load(f)
		for slide in json_contents['slides']:
			keys = ['p', 'LA', 'L', 'entropy']
			vals = [parse_datafield(slide['data'][key]) for key in keys]
				
			data.add_dataslide(DataSlide(keys, vals))
	
	return data

def plot_run(data: DataFrame, run_id: int, average_interval: int = 1, ax = None):
	if ax is None:
		ax = plt.gca()

	entropy = data.get_property_with_id("entropy", run_id)
	if average_interval != 1:
		times = []
		entropy_avg = []
		for i in range(len(entropy)//average_interval):
			times.append(i*average_interval)
			entropy_avg.append(np.mean(entropy[i*average_interval:(i+1)*average_interval]))
	else:
		entropy_avg = entropy

	ax.plot(entropy_avg)
	ax.set_xlabel(r'$t$', fontsize=16)
	ax.set_ylabel(r'$S_A^2$', fontsize=16)

def plot_all_data(data: DataFrame, steady_state: int = 0, ax = None):
	assert steady_state < len(data.slides[0]['entropy']), "Steady state longer than total evolution time"
	if ax is None:
		ax = plt.gca()

	unique_p = sorted(list(set(data.get_property('p'))))
	unique_LA = sorted(list(set(data.get_property('LA'))))
	entropy_avg = np.zeros((len(unique_p), len(unique_LA)))
	for slide in data.slides:
		i = unique_p.index(slide['p'])
		j = unique_LA.index(slide['LA'])

		entropy_avg[i][j] = np.mean(slide['entropy'][steady_state:])

	colors = ['C0', 'orange', 'yellow', 'purple', 'green', 'black', 'magenta', 'cyan']
	for n,p in enumerate(unique_p):
		if p != 0:
			ax.plot(unique_LA, entropy_avg[n], linewidth=1.5, marker='*', color=colors[n-1], label=f'p = {p}')

	ax.legend(fontsize=16)
	ax.set_xlabel(r'$L_A$', fontsize=16)
	ax.set_ylabel(r'$\overline{S_A^{(2)}}$', fontsize=16)

def fig2(filenames):
	data = []
	for filename in filenames:
		data.append(load_data(filename))
	
	xs = {}
	Ss = {}
	for df in data:
		xs[df.slides[0]['L']] = []
		Ss[df.slides[0]['L']] = []
		for slide in df.slides:
			xs[slide['L']].append(slide['LA'])
			Ss[slide['L']].append(np.mean(slide['entropy']))
		xs[df.slides[0]['L']] = np.array(xs[df.slides[0]['L']])
		Ss[df.slides[0]['L']] = np.array(Ss[df.slides[0]['L']])
	
	for L, LA in xs.items():
		xs[L] = np.log(np.sin(np.pi*LA/L)*L/np.pi)

	for L, logx in xs.items():
		inds = np.argsort(xs[L])
		xs[L] = xs[L][inds]
		Ss[L] = Ss[L][inds]
		p = np.polyfit(xs[L][:20], Ss[L][:20], 1)
		print(f'{L}: {p}')
		#plt.plot(xs[L], p[0]*xs[L] + p[1], linestyle='--', label=f'L = {L} fit')
		plt.plot(xs[L], Ss[L], label=f'L = {L}')


	for slide in data[2].slides:
		if slide['LA'] == 400:
			St = np.array(slide['entropy'])
	N = 1000

	#St = np.convolve(St, np.ones(N)/N, mode='valid')
	t = np.arange(0, 100000, 5)	
	#t = np.convolve(t, np.ones(N)/N, mode='valid')

#	plt.plot(np.log(t), St)
	plt.xlabel(r'$\log(x)$', fontsize=16)
	plt.ylabel(r'$\overline{S_A^{(2)}}$', fontsize=16)
	plt.legend(fontsize=16)
	plt.show()


#data = load_data('data/base_short.json')
#print(f'num samples: {len(data.slides[0]["entropy"])}')
#plot_all_data(data, steady_state=0)
#plt.show()

filenames = ['data/fig2_1.json', 'data/fig2_2.json', 'data/fig2_3.json']
fig2(filenames)
