import { BrowserRouter as Router, Routes, Route, Link } from 'react-router-dom';
import { AppProvider, useAppContext } from './context/AppContext';
import { WalletConnect } from './components/WalletConnect';
import { ErrorDisplay } from './components/ErrorDisplay';
import { HomePage } from './pages/HomePage';
import { ProposalPage } from './pages/ProposalPage';
import { CreateProposalPage } from './pages/CreateProposalPage';
import { HistoryPage } from './pages/HistoryPage';
import './App.css';

function AppContent() {
  const { state, connectWallet, disconnectWallet } = useAppContext();

  return (
    <Router>
      <div className="App">
        <header>
          <nav>
            <Link to="/">Home</Link>
            <Link to="/create">Create Proposal</Link>
            <Link to="/history">Voting History</Link>
          </nav>
          <WalletConnect
            onConnect={connectWallet}
            onDisconnect={disconnectWallet}
            connectedAddress={state.wallet.address}
          />
        </header>

        <main>
          <ErrorDisplay
            error={state.transaction.error}
            autoDismiss={true}
          />
          
          <Routes>
            <Route path="/" element={<HomePage />} />
            <Route
              path="/proposal/:id"
              element={<ProposalPage connectedAddress={state.wallet.address} />}
            />
            <Route path="/create" element={<CreateProposalPage />} />
            <Route
              path="/history"
              element={<HistoryPage connectedAddress={state.wallet.address} />}
            />
          </Routes>
        </main>
      </div>
    </Router>
  );
}

function App() {
  return (
    <AppProvider>
      <AppContent />
    </AppProvider>
  );
}

export default App;
