import 'package:flutter/material.dart';
import 'package:fl_chart/fl_chart.dart';
import 'package:provider/provider.dart';
import '../models/app_state.dart';

/// Widget for displaying resource usage charts
class ResourceUsageChart extends StatelessWidget {
  final String title;
  final String subtitle;
  final double Function(AppState) valueGetter;
  final double maxValue;
  final String unit;
  final Color color;
  final List<Color> gradientColors;
  final IconData icon;

  const ResourceUsageChart({
    super.key,
    required this.title,
    required this.subtitle,
    required this.valueGetter,
    required this.maxValue,
    required this.unit,
    required this.color,
    required this.gradientColors,
    required this.icon,
  });

  @override
  Widget build(BuildContext context) {
    return Consumer<AppState>(
      builder: (context, appState, child) {
        final value = valueGetter(appState);
        final percentage = (value / maxValue * 100).clamp(0, 100);

        return Card(
          elevation: 2,
          child: Padding(
            padding: const EdgeInsets.all(16.0),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Row(
                  children: [
                    Icon(icon, color: color),
                    const SizedBox(width: 8),
                    Text(
                      title,
                      style: const TextStyle(
                        fontSize: 16,
                        fontWeight: FontWeight.bold,
                      ),
                    ),
                  ],
                ),
                const SizedBox(height: 4),
                Text(
                  subtitle,
                  style: TextStyle(fontSize: 12, color: Colors.grey[600]),
                ),
                const SizedBox(height: 16),
                Row(
                  children: [
                    Text(
                      '$value $unit',
                      style: TextStyle(
                        fontSize: 20,
                        fontWeight: FontWeight.bold,
                        color: color,
                      ),
                    ),
                    const Spacer(),
                    Text(
                      '${percentage.toStringAsFixed(1)}%',
                      style: const TextStyle(
                        fontSize: 14,
                        fontWeight: FontWeight.bold,
                      ),
                    ),
                  ],
                ),
                const SizedBox(height: 8),
                SizedBox(
                  height: 100,
                  child: LineChart(
                    LineChartData(
                      gridData: FlGridData(show: false),
                      titlesData: FlTitlesData(show: false),
                      borderData: FlBorderData(show: false),
                      minX: 0,
                      maxX: 10,
                      minY: 0,
                      maxY: maxValue,
                      lineBarsData: [
                        LineChartBarData(
                          spots: [
                            FlSpot(0, value * 0.8),
                            FlSpot(2, value * 0.9),
                            FlSpot(4, value * 0.85),
                            FlSpot(6, value * 0.95),
                            FlSpot(8, value * 0.9),
                            FlSpot(10, value),
                          ],
                          isCurved: true,
                          gradient: LinearGradient(colors: gradientColors),
                          barWidth: 3,
                          isStrokeCapRound: true,
                          dotData: FlDotData(show: false),
                          belowBarData: BarAreaData(
                            show: true,
                            gradient: LinearGradient(
                              colors:
                                  gradientColors
                                      .map((color) => color.withOpacity(0.3))
                                      .toList(),
                            ),
                          ),
                        ),
                      ],
                    ),
                  ),
                ),
                const SizedBox(height: 8),
                LinearProgressIndicator(
                  value: percentage / 100,
                  backgroundColor: Colors.grey[200],
                  valueColor: AlwaysStoppedAnimation<Color>(color),
                ),
              ],
            ),
          ),
        );
      },
    );
  }
}

/// Widget for displaying a grid of resource usage charts
class ResourceUsageGrid extends StatelessWidget {
  const ResourceUsageGrid({super.key});

  @override
  Widget build(BuildContext context) {
    return GridView.count(
      crossAxisCount: 2,
      childAspectRatio: 1.2,
      crossAxisSpacing: 16,
      mainAxisSpacing: 16,
      shrinkWrap: true,
      physics: const NeverScrollableScrollPhysics(),
      children: [
        ResourceUsageChart(
          title: 'CPU Usage',
          subtitle: 'Percentage of CPU utilized by validation',
          valueGetter: (state) => state.cpuUsage,
          maxValue: 100,
          unit: '%',
          color: Colors.blue,
          gradientColors: [Colors.blue.shade300, Colors.blue.shade700],
          icon: Icons.memory,
        ),
        ResourceUsageChart(
          title: 'Memory Usage',
          subtitle: 'RAM utilized by validation process',
          valueGetter: (state) => state.memoryUsage,
          maxValue: 1000,
          unit: 'MB',
          color: Colors.purple,
          gradientColors: [Colors.purple.shade300, Colors.purple.shade700],
          icon: Icons.storage,
        ),
        ResourceUsageChart(
          title: 'Network Usage',
          subtitle: 'Bandwidth utilized for blockchain sync',
          valueGetter: (state) => state.networkUsage,
          maxValue: 5,
          unit: 'MB/s',
          color: Colors.green,
          gradientColors: [Colors.green.shade300, Colors.green.shade700],
          icon: Icons.network_check,
        ),
        ResourceUsageChart(
          title: 'Disk Usage',
          subtitle: 'Storage utilized for blockchain data',
          valueGetter: (state) => state.diskUsage,
          maxValue: 10,
          unit: 'GB',
          color: Colors.amber,
          gradientColors: [Colors.amber.shade300, Colors.amber.shade700],
          icon: Icons.storage,
        ),
      ],
    );
  }
}
