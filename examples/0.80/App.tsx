import { useEffect, useState } from 'react';
import {
  View,
  Text,
  TextInput,
  TouchableOpacity,
  StyleSheet,
  ScrollView,
  Image,
} from 'react-native';
import { add, subtract, multiply, divide } from 'craby-calculator';

const App = () => {
  const [a, setA] = useState('');
  const [b, setB] = useState('');
  const [results, setResults] = useState<{
    add: string | null;
    sub: string | null;
    mul: string | null;
    div: string | null;
  }>({
    add: null,
    sub: null,
    mul: null,
    div: null,
  });

  useEffect(() => {
    const n1 = parseFloat(a);
    const n2 = parseFloat(b);

    if (isNaN(n1) || isNaN(n2)) {
      console.warn('Invalid numbers');
      return;
    }

    setResults({
      add: add(n1, n2).toFixed(2),
      sub: subtract(n1, n2).toFixed(2),
      mul: multiply(n1, n2).toFixed(2),
      div: divide(n1, n2).toFixed(2),
    });
  }, [a, b]);

  const clear = () => {
    setA('');
    setB('');
    setResults({ add: null, sub: null, mul: null, div: null });
  };

  return (
    <ScrollView
      style={styles.container}
      contentContainerStyle={styles.contentContainer}
    >
      {/* React Logo */}
      <View style={styles.logoContainer}>
        <View style={styles.logo}>
          <Image style={styles.logo} source={require('./assets/react.png')} />
        </View>
      </View>

      {/* Title */}
      <Text style={styles.title}>Welcome to React Native</Text>

      {/* Description */}
      <Text style={styles.description}>
        Type-safe Rust for TurboModules—auto-generated, fully integrated
      </Text>

      {/* Input Section */}
      <View style={styles.inputCard}>
        <View style={styles.inputHeader}>
          <Text style={styles.inputTitle}>Enter Numbers</Text>
        </View>

        <TextInput
          style={styles.input}
          placeholder="First number"
          value={a}
          onChangeText={text => setA(text.replace(/[^0-9.]/g, ''))}
          keyboardType="numeric"
          placeholderTextColor="#999"
        />

        <TextInput
          style={styles.input}
          placeholder="Second number"
          value={b}
          onChangeText={text => setB(text.replace(/[^0-9.]/g, ''))}
          keyboardType="numeric"
          placeholderTextColor="#999"
        />

        <View style={styles.buttonContainer}>
          <TouchableOpacity style={styles.clearButton} onPress={clear}>
            <Text style={styles.clearButtonText}>Clear</Text>
          </TouchableOpacity>
        </View>
      </View>

      {/* Calculator Cards */}
      <CalculatorCard
        a={a}
        b={b}
        title="Addition"
        func="add"
        result={results.add}
        operation="+"
        color="#10B981"
      />

      <CalculatorCard
        a={a}
        b={b}
        title="Subtraction"
        func="sub"
        result={results.sub}
        operation="-"
        color="#3B82F6"
      />

      <CalculatorCard
        a={a}
        b={b}
        title="Multiplication"
        func="mul"
        result={results.mul}
        operation="×"
        color="#8B5CF6"
      />

      <CalculatorCard
        a={a}
        b={b}
        title="Division"
        func="div"
        result={results.div}
        operation="÷"
        color="#F59E0B"
      />

      {/* Footer */}
      <View style={styles.footer}>
        <Text style={styles.footerText}>Developed by @leegeunhyeok</Text>
      </View>
    </ScrollView>
  );
};

const CalculatorCard = ({
  a,
  b,
  title,
  func,
  result,
  operation,
  color,
}: {
  a: string;
  b: string;
  title: string;
  func: string;
  result: string | null;
  operation: string;
  color: string;
}) => (
  <View style={styles.card}>
    <View style={styles.cardContent}>
      <View style={styles.cardLeft}>
        <Text style={styles.cardTitle}>{title}</Text>
        <Text style={styles.cardSubtitle}>{func}(a, b)</Text>
      </View>
      <View style={styles.cardRight}>
        <Text style={[styles.cardResult, { color }]}>
          {result !== null ? result : '—'}
        </Text>
        <Text style={styles.cardOperation}>
          {result !== null ? `${a} ${operation} ${b}` : 'Result'}
        </Text>
      </View>
    </View>
  </View>
);

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#F3F4F6',
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
    height: 80,
    aspectRatio: 1,
    marginBottom: 24,
  },
  logoText: {
    fontSize: 60,
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
  inputCard: {
    width: '100%',
    backgroundColor: '#DBEAFE',
    borderRadius: 12,
    padding: 16,
    marginTop: 30,
    marginBottom: 20,
  },
  inputHeader: {
    flexDirection: 'row',
    alignItems: 'center',
    marginBottom: 15,
  },
  inputIcon: {
    fontSize: 20,
    marginRight: 8,
  },
  inputTitle: {
    fontSize: 16,
    fontWeight: '500',
    color: '#000',
  },
  input: {
    backgroundColor: '#FFF',
    borderRadius: 8,
    padding: 12,
    fontSize: 16,
    marginBottom: 12,
    borderWidth: 1,
    borderColor: '#D1D5DB',
  },
  buttonContainer: {
    flexDirection: 'row',
    gap: 10,
  },
  clearButton: {
    flex: 1,
    backgroundColor: '#FFF',
    borderRadius: 8,
    padding: 12,
    alignItems: 'center',
    paddingHorizontal: 20,
    borderWidth: 1,
    borderColor: '#D1D5DB',
  },
  clearButtonText: {
    color: '#374151',
    fontSize: 16,
    fontWeight: '500',
  },
  sectionTitle: {
    fontSize: 20,
    fontWeight: '500',
    marginBottom: 20,
    color: '#000',
  },
  card: {
    width: '100%',
    backgroundColor: '#FFF',
    borderRadius: 12,
    padding: 16,
    marginBottom: 12,
    shadowColor: '#000',
    shadowOffset: {
      width: 0,
      height: 1,
    },
    shadowOpacity: 0.1,
    shadowRadius: 2,
    elevation: 2,
    borderWidth: 1,
    borderColor: '#E5E7EB',
  },
  cardContent: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
  },
  cardLeft: {
    flex: 1,
  },
  cardTitle: {
    fontSize: 18,
    fontWeight: '500',
    color: '#000',
    marginBottom: 4,
  },
  cardSubtitle: {
    fontSize: 14,
    color: '#6B7280',
  },
  cardRight: {
    alignItems: 'flex-end',
  },
  cardResult: {
    fontSize: 24,
    fontWeight: 'bold',
    marginBottom: 4,
  },
  cardOperation: {
    fontSize: 12,
    color: '#6B7280',
  },
  footer: {
    marginTop: 20,
    paddingHorizontal: 20,
  },
  footerText: {
    fontSize: 14,
    color: '#6B7280',
    textAlign: 'center',
    fontFamily: 'monospace',
  },
});

export default App;
