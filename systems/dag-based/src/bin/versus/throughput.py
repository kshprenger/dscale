import matplotlib.pyplot as plt
import numpy as np
import pandas as pd

secs = 60

sb_2000 = pd.read_csv("sparse_bullshark_2000.csv", sep=" ", header=None)
avg_sb_2000 = sb_2000.groupby(0)[1].mean().reset_index()

plt.plot(
    avg_sb_2000[0],
    avg_sb_2000[1] / secs,
    "D-",
    color="red",
    label="Sparse Bullshark 2Gb/sec",
)

b_2000 = pd.read_csv("bullshark_2000.csv", sep=" ", header=None)
mean_b_2000 = b_2000[0].mean()

plt.axhline(
    y=mean_b_2000 / secs,
    color="red",
    linestyle="--",
    linewidth=2,
    label="Bullshark 2Gb/sec",
)


sb_4000 = pd.read_csv("sparse_bullshark_4000.csv", sep=" ", header=None)
avg_sb_4000 = sb_4000.groupby(0)[1].mean().reset_index()

plt.plot(
    avg_sb_4000[0],
    avg_sb_4000[1] / secs,
    "s-",
    color="green",
    label="Sparse Bullshark 4Gb/sec",
)

b_4000 = pd.read_csv("bullshark_4000.csv", sep=" ", header=None)
mean_b_4000 = b_4000[0].mean()

plt.axhline(
    y=mean_b_4000 / secs,
    color="green",
    linestyle="--",
    linewidth=2,
    label="Bullshark 4Gb/sec",
)

sb_6000 = pd.read_csv("sparse_bullshark_6000.csv", sep=" ", header=None)
avg_sb_6000 = sb_6000.groupby(0)[1].mean().reset_index()

plt.plot(
    avg_sb_6000[0],
    avg_sb_6000[1] / secs,
    "^-",
    color="blue",
    label="Sparse Bullshark 6Gb/sec",
)

b_6000 = pd.read_csv("bullshark_6000.csv", sep=" ", header=None)
mean_b_6000 = b_6000[0].mean()

plt.axhline(
    y=mean_b_6000 / secs,
    color="blue",
    linestyle="--",
    linewidth=2,
    label="Bullshark 6Gb/sec",
)

plt.xticks(avg_sb_2000[0])
y_min = 0
y_max = 7000
plt.yticks(np.arange(y_min, y_max + 500, 500))

plt.xlabel("Sample size")
plt.ylabel("Blocks per second")
plt.title("Throughput comparison")
plt.legend(bbox_to_anchor=(0.5, -0.15), loc="upper center", ncol=3)
plt.tight_layout()
plt.show()
