import { useEffect } from 'react';
import { BrowserRouter, Routes, Route } from 'react-router-dom';
import { useRealTimeSimulation } from '@/stores/strategyStore';
import { ToastProvider } from '@/contexts/ToastContext';
import PageLayout from '@/components/Layout/PageLayout';
import OverviewPage from '@/pages/OverviewPage';
import StrategyPage from '@/pages/StrategyPage';
import DashboardPage from '@/pages/DashboardPage';

function App() {
  const { startRealTimeSimulation } = useRealTimeSimulation();

  // Start simulation when app loads
  useEffect(() => {
    startRealTimeSimulation();
  }, []);

  return (
    <BrowserRouter>
      <ToastProvider>
        <PageLayout>
          <Routes>
            <Route path="/" element={<OverviewPage />} />
            <Route path="/strategies" element={<StrategyPage />} />
            <Route path="/dashboard" element={<DashboardPage />} />
          </Routes>
        </PageLayout>
      </ToastProvider>
    </BrowserRouter>
  );
}

export default App;