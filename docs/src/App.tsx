import { Routes, Route, Navigate } from 'react-router-dom'
import { Landing } from './pages/Landing'
import GettingStarted from './pages/GettingStarted'
import Configuration from './pages/Configuration'
import Builtins from './pages/Builtins'
import Scripting from './pages/Scripting'
import Parser from './pages/Parser'
import Globbing from './pages/Globbing'
import Autocomplete from './pages/Autocomplete'
import Prompt from './pages/Prompt'
import Jobs from './pages/Jobs'
import History from './pages/History'
import VsBash from './pages/VsBash'
import Examples from './pages/Examples'
import { Layout } from './components/Layout'

export default function App() {
  return (
    <Layout>
      <Routes>
        <Route path="/" element={<Navigate to="/docs" replace />} />
        <Route path="/docs" element={<Landing />} />
        <Route path="/docs/getting-started" element={<GettingStarted />} />
        <Route path="/docs/configuration" element={<Configuration />} />
        <Route path="/docs/builtins" element={<Builtins />} />
        <Route path="/docs/scripting" element={<Scripting />} />
        <Route path="/docs/parser" element={<Parser />} />
        <Route path="/docs/globbing" element={<Globbing />} />
        <Route path="/docs/autocomplete" element={<Autocomplete />} />
        <Route path="/docs/prompt" element={<Prompt />} />
        <Route path="/docs/jobs" element={<Jobs />} />
        <Route path="/docs/history" element={<History />} />
        <Route path="/docs/vs-bash" element={<VsBash />} />
        <Route path="/docs/examples" element={<Examples />} />
        <Route path="*" element={<Navigate to="/docs" replace />} />
      </Routes>
    </Layout>
  )
}
