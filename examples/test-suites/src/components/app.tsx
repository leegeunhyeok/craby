import { useState } from 'react';
import { Image, ScrollView, StyleSheet, Text, TouchableOpacity, View } from 'react-native';
import { ENCODED_LOGO } from '../assets/logo';
import { TEST_SUITES } from '../test-suites';
import { ResultCard } from './result-card';

export function App() {
  const [testResults, setTestResults] = useState<
    Array<{
      label: string;
      description?: string;
      result: any;
      error?: string;
    }>
  >([]);
  const [isRunning, setIsRunning] = useState(false);

  const runAllTests = async () => {
    setIsRunning(true);
    setTestResults([]);

    const results = [];
    for (const test of TEST_SUITES) {
      try {
        const result = await test.action();
        results.push({
          label: test.label,
          description: test.description,
          result: result,
        });
      } catch (error) {
        results.push({
          label: test.label,
          description: test.description,
          result: null,
          error: error instanceof Error ? error.message : String(error),
        });
      }
    }

    setTestResults(results);
    setIsRunning(false);
  };

  return (
    <ScrollView style={styles.container} contentContainerStyle={styles.contentContainer}>
      {/* Logo */}
      <View style={styles.logoContainer}>
        <View style={styles.logo}>
          <Image style={styles.logo} source={{ uri: ENCODED_LOGO }} />
        </View>
      </View>

      {/* Title */}
      <Text style={styles.title}>Test Suite Runner</Text>

      {/* Description */}
      <Text style={styles.description}>Run all test suites and view results</Text>

      {/* Run Test Button */}
      <View style={styles.buttonCard}>
        <TouchableOpacity
          style={[styles.runButton, isRunning && styles.runButtonDisabled]}
          onPress={runAllTests}
          disabled={isRunning}
        >
          <Text style={styles.runButtonText}>{isRunning ? 'Running Tests...' : 'Run All Tests'}</Text>
        </TouchableOpacity>
      </View>

      <View>
        <Text style={styles.testCountText}>
          Test suite passed: {testResults.filter((testResult) => !testResult.error).length} of {testResults.length}
        </Text>
      </View>

      {/* Test Results */}
      {testResults.map((testResult, index) => (
        <ResultCard
          key={`${testResult.label}-${index}`}
          label={testResult.label}
          description={testResult.description}
          result={testResult.result}
          error={testResult.error}
        />
      ))}
    </ScrollView>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#fff',
  },
  contentContainer: {
    alignItems: 'center',
    paddingHorizontal: 20,
    paddingTop: 60,
    paddingBottom: 40,
  },
  logoContainer: {
    marginTop: 64,
    marginBottom: 30,
  },
  logo: {
    width: 150,
    height: 100,
    resizeMode: 'contain',
    marginBottom: 24,
  },
  title: {
    fontSize: 28,
    fontWeight: '300',
    color: '#000',
    marginBottom: 10,
    textAlign: 'center',
  },
  description: {
    fontSize: 16,
    color: '#6B7280',
    marginBottom: 5,
    textAlign: 'center',
  },
  buttonCard: {
    width: '100%',
    marginTop: 30,
    marginBottom: 10,
  },
  runButton: {
    width: '100%',
    backgroundColor: '#387ca0',
    borderRadius: 8,
    padding: 16,
    alignItems: 'center',
  },
  runButtonDisabled: {
    backgroundColor: '#9CA3AF',
  },
  runButtonText: {
    color: '#FFF',
    fontSize: 16,
    fontWeight: '600',
  },
  testCountText: {
    fontSize: 14,
    color: '#6B7280',
    marginBottom: 8,
  },
});
