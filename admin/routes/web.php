<?php

use App\Http\Controllers\Web\ExamGatewayController;
use App\Http\Controllers\Web\StudentLoginController;
use Illuminate\Support\Facades\Route;

Route::get('/', [StudentLoginController::class, 'show'])
    ->middleware('examuq.client')
    ->name('student.login');

Route::post('/student/login', [StudentLoginController::class, 'submit'])
    ->middleware('examuq.client')
    ->name('student.login.submit');

Route::get('/exam-confirm', [ExamGatewayController::class, 'confirm'])
    ->middleware('launch.token')
    ->name('exam.confirm');

Route::get('/exam-gateway', [ExamGatewayController::class, 'redirectToExam'])
    ->middleware('launch.token')
    ->name('exam.gateway');

Route::get('/exam-blocked', [ExamGatewayController::class, 'blocked'])
    ->name('exam.block');
