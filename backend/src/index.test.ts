import request from 'supertest';
import app from './index';

describe('Backend API', () => {
  it('should return health status', async () => {
    const response = await request(app).get('/api/health');
    expect(response.status).toBe(200);
    expect(response.body).toHaveProperty('status', 'healthy');
  });
});
