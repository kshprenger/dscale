import matplotlib.pyplot as plt
import pandas as pd

sb_2000 = pd.read_csv("sparse_bullshark_2000.csv", sep=" ", header=None)
avg_sb_2000 = sb_2000.groupby(0)[3].mean().reset_index()
b_2000 = pd.read_csv("bullshark_2000.csv", sep=" ", header=None)
avg_b_2000 = b_2000.groupby(0)[2].mean().reset_index()[2].mean()
plt.plot(
    avg_sb_2000[0],
    (avg_sb_2000[3] * 8000) / (1024 * 1024),
    "D-",
    color="red",
    label="Sparse Bullshark 2Gb/sec",
)

plt.axhline(
    y=(avg_b_2000 * 8000) / (1024 * 1024),
    color="red",
    linestyle="--",
    linewidth=2,
    label="Bullshark 2Gb/sec",
)

sb_4000 = pd.read_csv("sparse_bullshark_4000.csv", sep=" ", header=None)
avg_sb_4000 = sb_4000.groupby(0)[3].mean().reset_index()
b_4000 = pd.read_csv("bullshark_4000.csv", sep=" ", header=None)
avg_b_4000 = b_4000.groupby(0)[2].mean().reset_index()[2].mean()
plt.plot(
    avg_sb_4000[0],
    (avg_sb_4000[3] * 8000) / (1024 * 1024),
    "s-",
    color="green",
    label="Sparse Bullshark 4Gb/sec",
)

plt.axhline(
    y=(avg_b_4000 * 8000) / (1024 * 1024),
    color="green",
    linestyle="--",
    linewidth=2,
    label="Bullshark 4Gb/sec",
)

sb_6000 = pd.read_csv("sparse_bullshark_6000.csv", sep=" ", header=None)
avg_sb_6000 = sb_6000.groupby(0)[3].mean().reset_index()
b_6000 = pd.read_csv("bullshark_6000.csv", sep=" ", header=None)
avg_b_6000 = b_6000.groupby(0)[2].mean().reset_index()[2].mean()
plt.plot(
    avg_sb_6000[0],
    (avg_sb_6000[3] * 8000) / (1024 * 1024),
    "^-",
    color="blue",
    label="Sparse Bullshark 6Gb/sec",
)

plt.axhline(
    y=(avg_b_6000 * 8000) / (1024 * 1024),
    color="blue",
    linestyle="--",
    linewidth=2,
    label="Bullshark 6Gb/sec",
)

plt.xticks(avg_sb_2000[0])

plt.xlabel("Sample size")
plt.ylabel("Network card saturation Mb/sec")
plt.title("Network card saturation comparison")
plt.legend(bbox_to_anchor=(0.5, -0.15), loc="upper center", ncol=3)
plt.tight_layout()
plt.show()
