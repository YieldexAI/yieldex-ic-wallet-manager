import React from 'react';
import { motion } from 'framer-motion';
import { useLocation } from 'react-router-dom';
import MainNavigation from '@/components/Navigation/MainNavigation';
import Layout, { Container } from '@/components/UI/Layout';
import { pageVariants } from '@/utils/animations';

interface PageLayoutProps {
  children: React.ReactNode;
  showNavigation?: boolean;
}

const PageLayout: React.FC<PageLayoutProps> = ({
  children,
  showNavigation = true
}) => {
  const location = useLocation();

  const getPageTitle = () => {
    switch (location.pathname) {
      case '/':
        return 'Overview';
      case '/strategies':
        return 'Build Your Strategy';
      case '/dashboard':
        return 'Dashboard';
      default:
        return 'Yieldex';
    }
  };

  return (
    <Layout maxWidth="xl">
      {showNavigation && <MainNavigation />}

      <motion.div
        key={location.pathname}
        variants={pageVariants}
        initial="initial"
        animate="enter"
        exit="exit"
        className="min-h-screen"
      >
        {/* Page Content */}
        <main className="flex-1">
          <div className="py-8">
            {children}
          </div>
        </main>

        {/* Footer */}
        <footer className="border-t border-gray-800 py-8">
          <Container>
            <div className="text-center space-y-4">
              <p className="text-gray-500 text-sm">
                Built with ❤️ on Internet Computer • Demo Interface
              </p>
              <div className="flex justify-center space-x-4 text-gray-600">
                <a
                  href="https://internetcomputer.org"
                  className="hover:text-primary-400 transition-colors text-sm"
                  target="_blank"
                  rel="noopener noreferrer"
                >
                  IC Docs
                </a>
                <span className="text-sm">•</span>
                <a
                  href="https://github.com"
                  className="hover:text-primary-400 transition-colors text-sm"
                  target="_blank"
                  rel="noopener noreferrer"
                >
                  GitHub
                </a>
                <span className="text-sm">•</span>
                <a
                  href="https://twitter.com"
                  className="hover:text-primary-400 transition-colors text-sm"
                  target="_blank"
                  rel="noopener noreferrer"
                >
                  Twitter
                </a>
              </div>
            </div>
          </Container>
        </footer>
      </motion.div>
    </Layout>
  );
};

export default PageLayout;