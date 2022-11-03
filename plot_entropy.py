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

def load_data(filename):
	data = DataFrame()
	with open(filename, 'r') as f:
		json_contents = json.load(f)
		for slide in json_contents['slides']:
			keys = list(slide['int_params'].keys()) + list(slide['float_params'].keys()) + list(slide['data'].keys())
			vals = []
			for key in keys:
				if key in slide['int_params']:
					vals.append(slide['int_params'][key])
				elif key in slide['float_params']:
					vals.append(slide['float_params'][key])
				else:
					vals.append(slide['data'][key])
				
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
	if ax is None:
		ax = plt.gca()

	unique_p = sorted(list(set(data.get_property('p'))))
	unique_LA = sorted(list(set(data.get_property('LA'))))
	entropy_avg = np.zeros((len(unique_p), len(unique_LA)))
	for slide in data.slides:
		print(slide.data)
		i = unique_p.index(slide['p'])
		j = unique_LA.index(slide['LA'])

		entropy_avg[i][j] = np.mean(slide['entropy'][steady_state:])

	colors = ['blue', 'orange', 'yellow', 'purple', 'green', 'black', 'magenta', 'cyan']
	for n,p in enumerate(unique_p):
		ax.plot(unique_LA, entropy_avg[n], linewidth=1.5, marker='*', color=colors[n], label=f'p = {p}')

	ax.legend(fontsize=16)
	ax.set_xlabel(r'$L_A$', fontsize=16)
	ax.set_ylabel(r'$\overline{S_A^{(2)}}$', fontsize=16)

data = load_data('data_test.json')

#unique_p = sorted(list(set(data.get_property('p'))))
#for p in unique_p:
#	p_data = data.query_key('p', 0.08)
#
#	fig, ax = plt.subplots()
#	for i in range(len(p_data.dfs)):
#		plot_run(data, i, 3000, ax)
#	ax.set_title(f'p = {p}', fontsize=16)
#	plt.show()

#plot_run(data, 50, 3000)
print(f'num samples: {len(data.slides[0]["entropy"])}')
plot_all_data(data, steady_state=0)
plt.show()
