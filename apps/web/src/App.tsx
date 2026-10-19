import React from 'react'
import { Routes, Route } from 'react-router-dom'
import { Sidebar } from './components/Sidebar'
import { Header } from './components/Header'
import { HomePage } from './features/home/HomePage'
import { LibraryPage } from './features/library/LibraryPage'

export const App: React.FC = () => {
  return (
    <div className="flex min-h-screen bg-bg">
      <Sidebar />
      <div className="flex-1 flex flex-col min-w-0">
        <Header />
        <main className="flex-1 overflow-y-auto">
          <Routes>
            <Route path="/" element={<HomePage />} />
            <Route path="/library/:kind" element={<LibraryPage />} />
          </Routes>
        </main>
      </div>
    </div>
  )
}
