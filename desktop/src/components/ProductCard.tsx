import type { Product } from '../types'
import './ProductCard.css'

interface ProductCardProps {
  product: Product
}

const ProductCard = ({ product }: ProductCardProps) => {
  const renderStars = (score: number) => {
    const stars = []
    for (let i = 1; i <= 5; i++) {
      stars.push(
        <span key={i} className={i <= Math.floor(score) ? 'star filled' : 'star'}>
          ★
        </span>
      )
    }
    return stars
  }

  return (
    <div className="product-card">
      {/* Card Header */}
      <div className="product-header">
        <div className="product-category">{product.category}</div>
        <div className="product-actions">
          <button className="product-menu" title="More options">
            ⋮
          </button>
        </div>
      </div>

      {/* Product Info */}
      <div className="product-info">
        <h3 className="product-name">{product.name}</h3>
        <p className="product-description">{product.description}</p>
        <div className="product-developer">By {product.developer}</div>
      </div>

      {/* Product Details */}
      <div className="product-details">
        <div className="product-price">
          {product.isPremium ? (
            <span className="premium-badge">Premium</span>
          ) : (
            <span className="wallet-badge">Wallet</span>
          )}
        </div>
        
        <div className="product-rating">
          <div className="stars">{renderStars(product.rating.score)}</div>
          <span className="rating-count">({product.rating.count})</span>
        </div>
      </div>

      {/* Installation Info */}
      <div className="product-installs">
        <span className="installs-text">Installs: {product.installs.toLocaleString()}+</span>
      </div>

      {/* Action Button */}
      <button className="action-button">
        {product.actionText}
      </button>
    </div>
  )
}

export default ProductCard