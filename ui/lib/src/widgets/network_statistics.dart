import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:fl_chart/fl_chart.dart';
import '../models/app_state.dart';
import '../services/blockchain_service.dart';

/// Widget for displaying network statistics
class NetworkStatistics extends StatefulWidget {
  const NetworkStatistics({super.key});

  @override
  State<NetworkStatistics> createState() => _NetworkStatisticsState();
}

class _NetworkStatisticsState extends State<NetworkStatistics> {
  final BlockchainService _blockchainService = BlockchainService.instance;

  // Mock data for charts - in a real implementation, this would be populated
  // from actual blockchain data
  final List<FlSpot> _transactionSpots = List.generate(
    24,
    (index) => FlSpot(index.toDouble(), (index * 1.5 + 10 + (index % 5) * 2)),
  );

  final List<FlSpot> _peerSpots = List.generate(
    24,
    (index) => FlSpot(index.toDouble(), 5 + (index % 7)),
  );

  @override
  Widget build(BuildContext context) {
    return Card(
      elevation: 2,
      child: Padding(
        padding: const EdgeInsets.all(16.0),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            const Text(
              'Network Statistics',
              style: TextStyle(fontSize: 18, fontWeight: FontWeight.bold),
            ),
            const SizedBox(height: 16),
            Consumer<AppState>(
              builder: (context, appState, child) {
                return Column(
                  children: [
                    _buildNetworkOverview(appState),
                    const SizedBox(height: 16),
                    _buildTransactionChart(),
                    const SizedBox(height: 16),
                    _buildPeerChart(),
                  ],
                );
              },
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildNetworkOverview(AppState appState) {
    return Row(
      mainAxisAlignment: MainAxisAlignment.spaceAround,
      children: [
        _buildStatCard(
          'Connected Peers',
          '${appState.connectedPeers}',
          Icons.people,
          Colors.blue,
        ),
        _buildStatCard(
          'Validated Transactions',
          '${appState.transactionsValidated}',
          Icons.receipt_long,
          Colors.green,
        ),
        FutureBuilder<Map<String, dynamic>>(
          future: _blockchainService.getNetworkStats(),
          builder: (context, snapshot) {
            final blockHeight =
                snapshot.hasData ? snapshot.data!['blockHeight'] : '...';
            return _buildStatCard(
              'Block Height',
              '$blockHeight',
              Icons.layers,
              Colors.purple,
            );
          },
        ),
      ],
    );
  }

  Widget _buildStatCard(
    String title,
    String value,
    IconData icon,
    Color color,
  ) {
    return Card(
      elevation: 1,
      child: Padding(
        padding: const EdgeInsets.symmetric(vertical: 12.0, horizontal: 16.0),
        child: Column(
          children: [
            Icon(icon, color: color, size: 24),
            const SizedBox(height: 8),
            Text(
              value,
              style: TextStyle(
                fontSize: 20,
                fontWeight: FontWeight.bold,
                color: color,
              ),
            ),
            const SizedBox(height: 4),
            Text(
              title,
              style: const TextStyle(fontSize: 12, color: Colors.grey),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildTransactionChart() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        const Text(
          'Transaction Volume (24h)',
          style: TextStyle(fontSize: 16, fontWeight: FontWeight.bold),
        ),
        const SizedBox(height: 8),
        SizedBox(
          height: 150,
          child: LineChart(
            LineChartData(
              gridData: FlGridData(
                show: true,
                drawVerticalLine: false,
                horizontalInterval: 20,
                getDrawingHorizontalLine: (value) {
                  return FlLine(color: Colors.grey.shade300, strokeWidth: 1);
                },
              ),
              titlesData: FlTitlesData(
                show: true,
                rightTitles: AxisTitles(
                  sideTitles: SideTitles(showTitles: false),
                ),
                topTitles: AxisTitles(
                  sideTitles: SideTitles(showTitles: false),
                ),
                bottomTitles: AxisTitles(
                  sideTitles: SideTitles(
                    showTitles: true,
                    reservedSize: 30,
                    interval: 6,
                    getTitlesWidget: (value, meta) {
                      final hour = value.toInt() % 24;
                      return SideTitleWidget(
                        axisSide: meta.axisSide,
                        child: Text('${hour}h'),
                      );
                    },
                  ),
                ),
                leftTitles: AxisTitles(
                  sideTitles: SideTitles(
                    showTitles: true,
                    interval: 20,
                    getTitlesWidget: (value, meta) {
                      return SideTitleWidget(
                        axisSide: meta.axisSide,
                        child: Text(value.toInt().toString()),
                      );
                    },
                    reservedSize: 42,
                  ),
                ),
              ),
              borderData: FlBorderData(
                show: true,
                border: Border.all(color: Colors.grey.shade300),
              ),
              minX: 0,
              maxX: 23,
              minY: 0,
              maxY: 100,
              lineBarsData: [
                LineChartBarData(
                  spots: _transactionSpots,
                  isCurved: true,
                  gradient: LinearGradient(
                    colors: [Colors.green.shade300, Colors.green.shade700],
                  ),
                  barWidth: 3,
                  isStrokeCapRound: true,
                  dotData: FlDotData(show: false),
                  belowBarData: BarAreaData(
                    show: true,
                    gradient: LinearGradient(
                      colors: [
                        Colors.green.shade300.withOpacity(0.3),
                        Colors.green.shade700.withOpacity(0.0),
                      ],
                      begin: Alignment.topCenter,
                      end: Alignment.bottomCenter,
                    ),
                  ),
                ),
              ],
            ),
          ),
        ),
      ],
    );
  }

  Widget _buildPeerChart() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        const Text(
          'Connected Peers (24h)',
          style: TextStyle(fontSize: 16, fontWeight: FontWeight.bold),
        ),
        const SizedBox(height: 8),
        SizedBox(
          height: 150,
          child: LineChart(
            LineChartData(
              gridData: FlGridData(
                show: true,
                drawVerticalLine: false,
                horizontalInterval: 5,
                getDrawingHorizontalLine: (value) {
                  return FlLine(color: Colors.grey.shade300, strokeWidth: 1);
                },
              ),
              titlesData: FlTitlesData(
                show: true,
                rightTitles: AxisTitles(
                  sideTitles: SideTitles(showTitles: false),
                ),
                topTitles: AxisTitles(
                  sideTitles: SideTitles(showTitles: false),
                ),
                bottomTitles: AxisTitles(
                  sideTitles: SideTitles(
                    showTitles: true,
                    reservedSize: 30,
                    interval: 6,
                    getTitlesWidget: (value, meta) {
                      final hour = value.toInt() % 24;
                      return SideTitleWidget(
                        axisSide: meta.axisSide,
                        child: Text('${hour}h'),
                      );
                    },
                  ),
                ),
                leftTitles: AxisTitles(
                  sideTitles: SideTitles(
                    showTitles: true,
                    interval: 5,
                    getTitlesWidget: (value, meta) {
                      return SideTitleWidget(
                        axisSide: meta.axisSide,
                        child: Text(value.toInt().toString()),
                      );
                    },
                    reservedSize: 42,
                  ),
                ),
              ),
              borderData: FlBorderData(
                show: true,
                border: Border.all(color: Colors.grey.shade300),
              ),
              minX: 0,
              maxX: 23,
              minY: 0,
              maxY: 20,
              lineBarsData: [
                LineChartBarData(
                  spots: _peerSpots,
                  isCurved: true,
                  gradient: LinearGradient(
                    colors: [Colors.blue.shade300, Colors.blue.shade700],
                  ),
                  barWidth: 3,
                  isStrokeCapRound: true,
                  dotData: FlDotData(show: false),
                  belowBarData: BarAreaData(
                    show: true,
                    gradient: LinearGradient(
                      colors: [
                        Colors.blue.shade300.withOpacity(0.3),
                        Colors.blue.shade700.withOpacity(0.0),
                      ],
                      begin: Alignment.topCenter,
                      end: Alignment.bottomCenter,
                    ),
                  ),
                ),
              ],
            ),
          ),
        ),
      ],
    );
  }
}
