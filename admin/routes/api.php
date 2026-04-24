<?php

use App\Http\Controllers\Api\LaunchController;
use App\Http\Controllers\Api\MonitoringController;
use App\Http\Controllers\Api\SessionController;
use Illuminate\Support\Facades\Route;

Route::prefix('v1')->group(function (): void {
    Route::post('/launch/request', [LaunchController::class, 'requestLaunch'])
        ->middleware('examuq.client');

    Route::post('/sessions/start', [SessionController::class, 'start']);
    Route::post('/sessions/{sessionId}/heartbeat', [SessionController::class, 'heartbeat']);
    Route::post('/sessions/{sessionId}/end', [SessionController::class, 'end']);

    Route::get('/monitoring/exams/{examId}/sessions', [MonitoringController::class, 'sessionsByExam']);
});
